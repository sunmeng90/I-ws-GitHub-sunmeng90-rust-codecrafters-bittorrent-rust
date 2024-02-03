use std::{env, string};
use std::collections::BTreeMap;

use serde::Serialize;

// Available if you need it!
use crate::Bencode::String;

#[derive(Debug, Serialize)]
enum Bencode {
    String(string::String),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<string::String, Bencode>),
}

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
fn decode_bencoded_value(encoded_value: &str) -> (Bencode, &str) {
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
                let (val, remainder) = decode_bencoded_value(rest);
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
                let (key, remainder) = decode_bencoded_value(rest);
                let key = match key {
                    String(s) => s,
                    s => panic!("dict key must be string, not {:?}", s)
                };
                let (value, remainder) = decode_bencoded_value(remainder);
                dict.insert(key.to_string(), value);
                rest = remainder;
            }
            return (Bencode::Dict(dict), &rest[1..]);
        }
        _ => {}
    }
    panic!("Unhandled encoded value: {}", encoded_value)
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<string::String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_val: serde_json::Value = decode_bencoded_value(encoded_value).0.try_into().unwrap();
        println!("{}", serde_json::to_string(&decoded_val).unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
