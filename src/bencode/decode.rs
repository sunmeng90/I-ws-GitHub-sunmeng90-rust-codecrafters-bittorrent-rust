use std::collections::BTreeMap;
use std::string;
use crate::bencode::Bencode;
use crate::bencode::Bencode::String;

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
