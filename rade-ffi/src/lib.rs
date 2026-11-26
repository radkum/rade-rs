use std::{
    ffi::{CStr, CString, c_char},
    panic::{AssertUnwindSafe, catch_unwind},
    path::Path,
    ptr, slice,
};

use rade_core::{Event, Events, Matches, RadeEngine, RuleSet, Rules};
use serde_json::{Value, json};

const RADE_ABI_VERSION: u32 = 1;
const RADE_OK: i32 = 0;
const RADE_ERR: i32 = 1;
const RADE_PANIC: i32 = 2;

pub struct RadeEngineHandle {
    engine: RadeEngine,
}

fn panic_message(payload: Box<dyn core::any::Any + Send>) -> String {
    if let Some(text) = payload.downcast_ref::<&str>() {
        (*text).to_string()
    } else if let Some(text) = payload.downcast_ref::<String>() {
        text.clone()
    } else {
        "unknown panic".to_string()
    }
}

fn sanitize(text: &str) -> String {
    text.chars()
        .map(|c| if c == '\0' { ' ' } else { c })
        .collect()
}

fn write_string(out: *mut *mut c_char, value: String) {
    if out.is_null() {
        return;
    }
    if let Ok(c_string) = CString::new(sanitize(&value)) {
        unsafe {
            *out = c_string.into_raw();
        }
    }
}

fn set_error(err: *mut *mut c_char, message: impl Into<String>) {
    write_string(err, message.into());
}

fn guard(err: *mut *mut c_char, f: impl FnOnce() -> i32) -> i32 {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(code) => code,
        Err(payload) => {
            set_error(err, format!("panic: {}", panic_message(payload)));
            RADE_PANIC
        },
    }
}

fn engine_from_rules(rules: Rules) -> RadeEngine {
    let mut engine = RadeEngine::from_rules(rules);
    engine.compile_rules();
    engine
}

fn matches_to_json(engine: &RadeEngine, matches: &Matches) -> Value {
    let mut event_name: Option<String> = None;
    let mut rules = Vec::new();

    for (event, matched_rules) in matches.iter() {
        if event_name.is_none() {
            event_name = event.name().map(ToString::to_string);
        }
        for rule_id in matched_rules.iter() {
            let rule_name = engine
                .rules()
                .get(rule_id)
                .and_then(|rule| rule.name())
                .map(ToString::to_string);
            rules.push(json!({
                "id": rule_id.to_string(),
                "name": rule_name,
            }));
        }
    }

    json!({
        "matched": !rules.is_empty(),
        "event": event_name,
        "rules": rules,
    })
}

/// ABI version implemented by this library.
#[unsafe(no_mangle)]
pub extern "C" fn rade_abi_version() -> u32 {
    RADE_ABI_VERSION
}

/// Create an engine from a compiled binary ruleset.
///
/// # Safety
///
/// `data` must point to `len` readable bytes for the duration of the call.
/// `err`, if non-null, must point to a writable `char *` slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rade_engine_new_from_ruleset(
    data: *const u8,
    len: usize,
    err: *mut *mut c_char,
) -> *mut RadeEngineHandle {
    if data.is_null() {
        set_error(err, "ruleset pointer is null");
        return ptr::null_mut();
    }

    let mut bytes = unsafe { slice::from_raw_parts(data, len) }.to_vec();

    match catch_unwind(AssertUnwindSafe(
        || -> Result<RadeEngine, String> {
            let rule_set =
                RuleSet::deserialize_from_bytes(&mut bytes).map_err(|e| e.to_string())?;
            Ok(engine_from_rules(rule_set.retain_rules()))
        },
    )) {
        Ok(Ok(engine)) => Box::into_raw(Box::new(RadeEngineHandle { engine })),
        Ok(Err(message)) => {
            set_error(err, message);
            ptr::null_mut()
        },
        Err(payload) => {
            set_error(err, format!("panic: {}", panic_message(payload)));
            ptr::null_mut()
        },
    }
}

/// Create an engine from a directory or file of YAML rules.
///
/// # Safety
///
/// `path` must point to a NUL-terminated UTF-8 string. `err`, if non-null,
/// must point to a writable `char *` slot.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rade_engine_new_from_yaml_dir(
    path: *const c_char,
    err: *mut *mut c_char,
) -> *mut RadeEngineHandle {
    if path.is_null() {
        set_error(err, "rules path is null");
        return ptr::null_mut();
    }

    let path = match unsafe { CStr::from_ptr(path) }.to_str() {
        Ok(path) => path.to_string(),
        Err(e) => {
            set_error(err, format!("rules path is not valid UTF-8: {e}"));
            return ptr::null_mut();
        },
    };

    match catch_unwind(AssertUnwindSafe(
        || -> Result<RadeEngine, String> {
            let rules = Rules::from_dir(Path::new(&path)).map_err(|e| e.to_string())?;
            Ok(engine_from_rules(rules))
        },
    )) {
        Ok(Ok(engine)) => Box::into_raw(Box::new(RadeEngineHandle { engine })),
        Ok(Err(message)) => {
            set_error(err, message);
            ptr::null_mut()
        },
        Err(payload) => {
            set_error(err, format!("panic: {}", panic_message(payload)));
            ptr::null_mut()
        },
    }
}

/// Evaluate a single JSON event and return a JSON result object.
///
/// Returns `0` on success, `1` on a handled error and `2` on panic.
///
/// # Safety
///
/// `engine` must be a pointer returned by one of the `rade_engine_new_*`
/// functions and must not be used concurrently from multiple threads.
/// `event_json` must point to a NUL-terminated UTF-8 string. `out_matches_json`
/// and `err`, if non-null, must point to writable `char *` slots. Strings
/// written to the out-parameters must be released with `rade_string_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rade_engine_eval_json(
    engine: *mut RadeEngineHandle,
    event_json: *const c_char,
    out_matches_json: *mut *mut c_char,
    err: *mut *mut c_char,
) -> i32 {
    guard(err, || {
        if engine.is_null() {
            set_error(err, "engine pointer is null");
            return RADE_ERR;
        }
        if event_json.is_null() {
            set_error(err, "event pointer is null");
            return RADE_ERR;
        }
        if out_matches_json.is_null() {
            set_error(err, "out_matches_json pointer is null");
            return RADE_ERR;
        }

        let engine = unsafe { &(*engine).engine };
        let event_text = match unsafe { CStr::from_ptr(event_json) }.to_str() {
            Ok(text) => text,
            Err(e) => {
                set_error(err, format!("event is not valid UTF-8: {e}"));
                return RADE_ERR;
            },
        };

        let event: Event = match serde_json::from_str(event_text) {
            Ok(event) => event,
            Err(e) => {
                set_error(err, format!("invalid event JSON: {e}"));
                return RADE_ERR;
            },
        };

        let matches = match engine.eval_with_predicates(Events::new(vec![event])) {
            Ok(matches) => matches,
            Err(e) => {
                set_error(err, e.to_string());
                return RADE_ERR;
            },
        };

        write_string(out_matches_json, matches_to_json(engine, &matches).to_string());
        RADE_OK
    })
}

/// Free an engine handle created by this library.
///
/// # Safety
///
/// `engine` must be null or a pointer previously returned by
/// `rade_engine_new_from_ruleset` / `rade_engine_new_from_yaml_dir` that has not
/// already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rade_engine_free(engine: *mut RadeEngineHandle) {
    if engine.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(engine));
    }
}

/// Free a string returned by this library.
///
/// # Safety
///
/// `s` must be null or a pointer previously returned through an out-parameter
/// of this library that has not already been freed.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rade_string_free(s: *mut c_char) {
    if s.is_null() {
        return;
    }
    unsafe {
        drop(CString::from_raw(s));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::{CStr, CString};
    use std::ptr;

    const RULE_YAML: &str = r#"
id: 43025534-69e4-4e81-a78f-fad61111a7df
name: Encoded PowerShell Command
condition: |
  Image.ends_with('powershell.exe') && CommandLine.contains('-enc')
"#;

    fn ruleset_bytes() -> Vec<u8> {
        let rule = rade_core::Rule::from_yaml(RULE_YAML).unwrap();
        let rules = rade_core::Rules::from(vec![rule]);
        rade_core::RuleSet::from(rules)
            .serialize_to_bytes()
            .unwrap()
    }

    #[test]
    fn abi_version_is_exposed() {
        assert_eq!(rade_abi_version(), RADE_ABI_VERSION);
    }

    #[test]
    fn engine_is_send_and_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<RadeEngine>();
    }

    #[test]
    fn eval_json_returns_matched_rule() {
        let bytes = ruleset_bytes();
        let mut err = ptr::null_mut();
        let engine =
            unsafe { rade_engine_new_from_ruleset(bytes.as_ptr(), bytes.len(), &mut err) };
        assert!(!engine.is_null());

        let event = CString::new(
            r#"{"Image":"C:/Windows/System32/WindowsPowerShell/v1.0/powershell.exe","CommandLine":"powershell.exe -enc SGVsbG8="}"#,
        )
        .unwrap();

        let mut out = ptr::null_mut();
        let code = unsafe { rade_engine_eval_json(engine, event.as_ptr(), &mut out, &mut err) };
        assert_eq!(code, RADE_OK);

        let json = unsafe { CStr::from_ptr(out) }.to_str().unwrap();
        assert!(json.contains("\"matched\":true"));
        assert!(json.contains("Encoded PowerShell Command"));

        unsafe {
            rade_string_free(out);
            rade_engine_free(engine);
        }
    }

    #[test]
    fn invalid_event_json_returns_error() {
        let bytes = ruleset_bytes();
        let mut err = ptr::null_mut();
        let engine =
            unsafe { rade_engine_new_from_ruleset(bytes.as_ptr(), bytes.len(), &mut err) };
        assert!(!engine.is_null());

        let event = CString::new("{not json}").unwrap();
        let mut out = ptr::null_mut();
        let code = unsafe { rade_engine_eval_json(engine, event.as_ptr(), &mut out, &mut err) };
        assert_eq!(code, RADE_ERR);
        assert!(out.is_null());

        let message = unsafe { CStr::from_ptr(err) }.to_str().unwrap();
        assert!(message.contains("invalid event JSON"));

        unsafe {
            rade_string_free(err);
            rade_engine_free(engine);
        }
    }
}
