use std::collections::{BTreeMap, HashMap};
use serde_json;
use std::env;
use std::iter::Map;

// Available if you need it!
use serde_bencode;
use serde_json::Value;

enum Bencode {
    String(String),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<Vec<u8>, Bencode>),
}

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> (serde_json::Value, &str) {
    match encoded_value.chars().next() {
        Some('0'..='9') => {
            if let Some((len, rest)) = encoded_value.split_once(':') {
                if let Ok(len) = len.parse::<usize>() {
                    return (rest[..len].to_string().into(), &rest[len..]);
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
                return (n.into(), rest);
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
            return (values.into(), &rest[1..]);
        }
        // d3:cow3:moo4:spam4:eggse
        Some('d') => {
            let mut dict: serde_json::Map<String, Value> = serde_json::Map::new();
            let mut rest: &str = encoded_value
                .split_at(1)
                .1;

            loop {
                if rest.starts_with('e') {
                    break;
                }
                let (key, remainder) = decode_bencoded_value(rest);
                let key = match key {
                    Value::String(s) => s,
                    s => panic!("dict key must be string, not {:?}", s)
                };
                let (value, remainder) = decode_bencoded_value(remainder);
                dict.insert(key.to_string().into(), value);
                rest = remainder;
            }
            return (dict.into(), &rest[1..]);
        }
        _ => {}
    }
    panic!("Unhandled encoded value: {}", encoded_value)
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value).0;
        println!("{}", serde_json::to_string(&decoded_value).unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
