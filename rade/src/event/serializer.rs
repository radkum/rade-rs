use serde::Serialize;
use crate::event::HashMap;

use super::{Event, FatString};

#[derive(serde::Deserialize, Serialize)]
struct EventSerialized(HashMap<String, SerializedVal>);

#[derive(serde::Deserialize, Serialize)]
enum SerializedVal {
    String(String),
    Number(u64),
}

impl<'de> serde::Deserialize<'de> for Event {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let serialized = EventSerialized::deserialize(deserializer)?;
        Ok(Event::from(&serialized))
    }
}

impl From<&Event> for EventSerialized {
    fn from(event: &Event) -> Self {
        let mut map: HashMap<String, SerializedVal> = HashMap::new();
        for (key, value) in &event.numbers {
            // Serialize each number field
            map.insert(key.clone(), SerializedVal::Number(*value));
        }
        for (key, value) in &event.strings {
            // Serialize each string field
            map.insert(key.clone(), SerializedVal::String(value.plain.clone()));
        }
        Self(map)
    }
}

impl From<&EventSerialized> for Event {
    fn from(serialized: &EventSerialized) -> Self {
        let mut numbers = HashMap::new();
        let mut strings = HashMap::<String, FatString>::new();
        let mut string_lists = HashMap::<String, Vec<FatString>>::new();

        for (key, value) in serialized.0.iter() {
            if key == "content" {
                if let SerializedVal::String(s) = &value {
                    string_lists.insert(format!("{key}_tokens"), Event::tokenize(s));
                }
            }
            match value {
                SerializedVal::Number(n) => {let _ = numbers.insert(key.to_string(), *n);},
                SerializedVal::String(s) => {let _ = strings.insert(key.to_string(), FatString::from(s));},
            };
        }
        Event {
            numbers,
            strings,
            string_lists,
        }
    }
}

impl serde::Serialize for Event {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let val_ser = EventSerialized::from(self);
        val_ser.serialize(serializer)
    }
}
