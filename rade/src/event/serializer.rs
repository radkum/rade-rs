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
        let mut bools = HashMap::new();
        let mut numbers = HashMap::new();
        let mut strings = HashMap::<String, FatString>::new();
        let mut string_lists = HashMap::<String, Vec<FatString>>::new();
        let mut number_lists = HashMap::<String, Vec<i64>>::new();

        for (key, value) in serialized.0.iter() {
            //println!("Processing key: {}, value: {:?}", key, value);
            match value {
                YamlValue::Bool(n) => {
                    let _ = bools.insert(key.to_string(), *n);
                },
                YamlValue::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        let _ = numbers.insert(key.to_string(), i);
                    }
                },
                YamlValue::String(s) => {
                    let _ = strings.insert(key.to_string(), FatString::from(s));
                },
                YamlValue::Sequence(seq) => {
                    let mut all_numbers = true;
                    for item in seq.iter() {
                        let YamlValue::Number(_) = item else {
                            all_numbers = false;
                            break;
                        };
                    }
                    if all_numbers {
                        let int_list = seq
                            .iter()
                            .filter_map(|item| item.as_i64())
                            .collect::<Vec<i64>>();
                        let _ = number_lists.insert(key.to_string(), int_list);
                    } else {
                        let str_list = seq
                            .iter()
                            .filter_map(|item| {
                                item.as_str().map(|s| FatString::from(s.to_string()))
                            })
                            .collect::<Vec<FatString>>();
                        let _ = string_lists.insert(key.to_string(), str_list);
                    }
                },
                _ => todo!(),
            };
            let _ = all.insert(key.to_string(), value.into());
        }
        Event {
            bools,
            numbers,
            strings,
            string_lists,
            number_lists,
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
