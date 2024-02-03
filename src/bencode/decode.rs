use std::collections::BTreeMap;
use std::string;
use crate::bencode::Bencode;
use crate::bencode::Bencode::String;

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


#[allow(dead_code)]
pub fn decode(encoded_value: &str) -> (Bencode, &str) {
    match encoded_value.chars().next() {
        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                if let Ok(len) = len.parse::<usize>() {
                    return (String(rest[..len].to_string()), &rest[len..]);
                }
            }
        }
        Some('i') => {
            if let Some((n, rest)) =
                encoded_value
                    .split_at(1)
                    .1
                    .split_once('e')
                    .and_then(|(digits, rest)| {
                        let n = digits.parse::<i64>().ok()?;
                        Some((n, rest))
                    }) {
                return (Bencode::Integer(n), rest);
            }
        }
        Some('l') => {
            let mut values = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.starts_with('e') {
                let (val, remainder) = decode(rest);
                values.push(val);
                rest = remainder;
            }
            return (Bencode::List(values), &rest[1..]);
        }
        // d3:cow3:moo4:spam4:eggse
        Some('d') => {
            let mut dict: BTreeMap<string::String, Bencode> = BTreeMap::new();
            let mut rest: &str = encoded_value.split_at(1).1;

            loop {
                if rest.starts_with('e') {
                    break;
                }
                let (key, remainder) = decode(rest);
                let key = match key {
                    String(s) => s,
                    s => panic!("dict key must be string, not {:?}", s)
                };
                let (value, remainder) = decode(remainder);
                dict.insert(key.to_string(), value);
                rest = remainder;
            }
            return (Bencode::Dict(dict), &rest[1..]);
        }
        _ => {}
    }
    panic!("Unhandled encoded value: {}", encoded_value)
}
