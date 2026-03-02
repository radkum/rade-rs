mod serializer;
use std::{collections::HashMap, fs::read_to_string};

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
pub use serializer::EventSerialized;

use crate::{FatString, RadeResult, Val};

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Events(Vec<Event>);
impl Events {
    pub fn iter(&self) -> core::slice::Iter<'_, Event> {
        self.0.iter()
    }

    pub fn get(self) -> Vec<Event> {
        self.0
    }

    fn add_event(&mut self, event: Event) {
        self.0.push(event);
    }

    #[cfg(feature = "std")]
    pub fn from_dir(path: &std::path::Path) -> RadeResult<Self> {
        fn imp_from_dir(path: &std::path::Path, events: &mut Events) -> RadeResult<()> {
            if path.is_file() {
                let content = read_to_string(path)
                    .map_err(|err| anyhow!("Failed to read file {}: {:?}", path.display(), err))?;
                let event_serialized = serde_yaml_bw::from_str::<EventSerialized>(&content)?;
                let mut event = Event::from(event_serialized);
                if event.name().is_none() {
                    event.set_name(path.file_stem().unwrap().to_string_lossy().as_ref());
                }
                events.add_event(event);
            } else if path.is_dir() {
                let events_dir = std::fs::read_dir(path)?;
                for event_dir_entry in events_dir {
                    let Ok(event) = event_dir_entry else {
                        log::warn!("Failed to read dir entry from path",);
                        continue;
                    };

                    if let Err(err) = imp_from_dir(&event.path(), events) {
                        println!(
                            "Failed to read event from path: {:?}, error: {:?}",
                            event.path(),
                            err
                        );
                    }
                }
            } else {
                Err(anyhow::anyhow!(
                    "Path {} is neither file nor directory",
                    path.display()
                ))?;
            }
            Ok(())
        }
        let mut events = Events::default();
        imp_from_dir(path, &mut events)?;
        Ok(events)
    }
}

impl Events {
    pub fn new(events: Vec<Event>) -> Self {
        Self(events)
    }

    #[cfg(feature = "std")]
    pub fn serialize<W: std::io::Write>(&self, out: W) -> RadeResult<()> {
        // let data = self.serialize_to_bytes()?;
        // out.write_all(&data)?;
        Ok(serde_yaml_bw::to_writer(out, self)?)
    }

    // pub(crate) fn serialize_to_bytes(&self) -> Result<Vec<u8>> {
    //     Ok(bincode::serde::encode_to_vec(&self.0, BIN_CONFIG)?)
    // }

    #[cfg(feature = "std")]
    pub fn deserialize<R: std::io::Read>(io_reader: R) -> RadeResult<Self> {
        // let mut data = vec![];
        // let _size = io_reader.read_to_end(&mut data)?;
        // Self::deserialize_from_bytes(&mut data)
        Ok(serde_yaml_bw::from_reader(io_reader)?)
    }
}

#[derive(Debug, Clone)]
pub struct Event {
    numbers: HashMap<String, u64>,
    bools: HashMap<String, bool>,
    strings: HashMap<String, FatString>,
    string_lists: HashMap<String, Vec<FatString>>,
    number_lists: HashMap<String, Vec<u64>>,
    all: HashMap<String, Val>,
}

impl Event {
    pub fn tokenize(s: &str) -> Vec<FatString> {
        s.split(|c: char| !c.is_alphanumeric() && c != '\\' && c != '.' && c != '-')
            .filter(|s| !s.is_empty())
            .map(|s| FatString::from(s.to_string()))
            .collect()
    }

    pub fn name(&self) -> Option<&str> {
        self.strings.get("name").map(|val| val.plain())
    }

    pub fn set_name(&mut self, name: &str) {
        self.strings
            .insert("name".to_string(), FatString::from(name.to_string()));
    }

    pub fn get_int_field(&self, field_name: &FatString) -> RadeResult<u64> {
        self.numbers.get(field_name.lowercased()).copied().ok_or_else(|| format!("Int field not found").into())
    }

    pub fn get_bool_field(&self, field_name: &FatString) -> RadeResult<bool> {
        self.bools.get(field_name.lowercased()).copied().ok_or_else(|| format!("Bool field not found").into())
    }

    pub fn get_str_field(&self, field_name: &FatString) -> RadeResult<&FatString> {
        self.strings.get(field_name.lowercased()).ok_or_else(|| format!("Str field not found").into())
    }

    pub fn get_strlist_field(&self, field_name: &FatString) -> RadeResult<&Vec<FatString>> {
        self.string_lists.get(field_name.lowercased()).ok_or_else(|| format!("Str list not found").into())
    }

    pub fn get_intlist_field(&self, field_name: &FatString) -> RadeResult<&Vec<u64>> {
        self.number_lists.get(field_name.lowercased()).ok_or_else(|| format!("Int list not found").into())
    }

    pub fn get_field(&self, field_name: &FatString) -> RadeResult<&Val> {
        self.all.get(field_name.lowercased()).ok_or_else(|| format!("Field not found").into())
    }
}
