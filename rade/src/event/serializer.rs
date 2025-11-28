use serde::Serialize;
use serde_yaml_bw::Value as YamlValue;

use super::{Event, FatString};
use crate::event::HashMap;

#[derive(serde::Deserialize, Serialize)]
pub struct EventSerialized(HashMap<String, YamlValue>);
impl EventSerialized {
    pub fn new(map: HashMap<String, YamlValue>) -> Self {
        EventSerialized(map)
    }
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
        Self(
            event
                .all
                .iter()
                .fold(
                    HashMap::<String, YamlValue>::new(),
                    |mut map, (key, val)| {
                        let _ = map.insert(key.clone(), val.into());
                        map
                    },
                )
                .into(),
        )
    }
}

impl From<EventSerialized> for Event {
    fn from(serialized: EventSerialized) -> Self {
        Event::from(&serialized)
    }
}
impl From<&EventSerialized> for Event {
    fn from(serialized: &EventSerialized) -> Self {
        let mut all = HashMap::new();
        let mut numbers = HashMap::new();
        let mut strings = HashMap::<String, FatString>::new();
        let mut string_lists = HashMap::<String, Vec<FatString>>::new();

        for (key, value) in serialized.0.iter() {
            if key == "content" {
                if let YamlValue::String(s) = &value {
                    string_lists.insert(format!("{key}_tokens"), Event::tokenize(s));
                }
            }
            match value {
                YamlValue::Number(n) => {
                    let _ = numbers.insert(key.to_string(), n.as_u64().unwrap_or_default());
                },
                YamlValue::String(s) => {
                    let _ = strings.insert(key.to_string(), FatString::from(s));
                },
                _ => todo!(),
            };
            let _ = all.insert(key.to_string(), value.into());
        }
        Event {
            numbers,
            strings,
            string_lists,
            all,
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
