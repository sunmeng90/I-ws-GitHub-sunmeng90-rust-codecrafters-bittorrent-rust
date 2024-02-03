use std::collections::BTreeMap;
use std::string;
use crate::bencode::Bencode;

impl TryFrom<Bencode> for serde_json::Value {
    type Error = &'static str;

    fn try_from(value: Bencode) -> Result<Self, Self::Error> {
        let val = match value {
            Bencode::String(s) => s.into(),
            Bencode::Integer(i) => serde_json::Value::from(i),
            Bencode::List(list) => Bencode::convert_list(&list),
            Bencode::Dict(dict) => Bencode::convert_dict(&dict)
        };
        Ok(val)
    }
}

impl Bencode {
    fn convert_list(list: &Vec<Bencode>) -> serde_json::Value {
        let val_list: Vec<serde_json::Value> = list.iter().map(|item| {
            match item {
                Bencode::String(s) => s.to_owned().into(),
                Bencode::Integer(i) => serde_json::Value::from(i.to_owned()),
                Bencode::List(l) => Bencode::convert_list(l),
                Bencode::Dict(d) => Bencode::convert_dict(d),
            }
        }).collect();
        serde_json::Value::from(val_list)
    }

    fn convert_dict(dict: &BTreeMap<string::String, Bencode>) -> serde_json::Value {
        let val_map: serde_json::Map<string::String, serde_json::Value> = dict.iter().map(|(k, v)| {
            let val = match v {
                Bencode::String(s) => s.to_owned().into(),
                Bencode::Integer(i) => serde_json::Value::from(i.clone()),
                Bencode::List(l) => Bencode::convert_list(l),
                Bencode::Dict(d) => Bencode::convert_dict(d),
            };
            (k.to_owned(), val)
        }).collect();
        serde_json::Value::from(val_map)
    }
}