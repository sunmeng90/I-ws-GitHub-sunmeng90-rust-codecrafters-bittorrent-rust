use std::collections::BTreeMap;
use std::string;
use crate::bencode::Bencode;
use crate::bencode::Bencode::{Byte};


fn split_once(content: &[u8], ch: u8) -> Option<(&[u8], &[u8])> {
    match content.iter().position(|u| *u == ch) {
        Some(p) => {
            Some((&content[..p], &content[p+1..]))
        }
        _ => None
    }
}

#[allow(dead_code)]
pub fn decode(encoded_value: &[u8]) -> (Bencode, &[u8]) {
    match encoded_value.iter().next() {
        Some(b'0'..=b'9') => {
            if let Some((len, rest)) = split_once(encoded_value, b':') {
                if let Ok(len) = string::String::from_utf8_lossy(len).parse::<usize>() {
                    let result = &rest[..len];
                    return (Byte(result.to_vec()), &rest[len..]);
                }
            }
        }
        Some(b'i') => {
            let content = encoded_value.split_at(1).1;
            if let Some((n, rest)) = split_once(content, b'e')
                .and_then(|(digits, rest)| {
                    let n = string::String::from_utf8_lossy(digits).parse::<i64>().ok()?;
                    Some((n, rest))
                }) {
                return (Bencode::Integer(n), rest);
            }
        }
        Some(b'l') => {
            let mut values = Vec::new();
            let mut rest = encoded_value.split_at(1).1;
            while !rest.is_empty() && !rest.first().filter(|c| *c == &b'e').is_some() {
                let (val, remainder) = decode(rest);
                values.push(val);
                rest = remainder;
            }
            return (Bencode::List(values), &rest[1..]);
        }
        // d3:cow3:moo4:spam4:eggse
        Some(b'd') => {
            let mut dict: BTreeMap<string::String, Bencode> = BTreeMap::new();
            let mut rest: &[u8] = encoded_value.split_at(1).1;
            loop {
                if rest.is_empty() || rest.first().filter(|c| *c == &b'e').is_some() {
                    break;
                }
                let (key, remainder) = decode(rest);
                let key = match key {
                    Byte(s) => s,
                    s => panic!("dict key must be string, not {:?}", s)
                };
                let (value, remainder) = decode(remainder);
                dict.insert(String::from_utf8(key).unwrap(), value);
                rest = remainder;
            }
            return (Bencode::Dict(dict), &rest[1..]);
        }
        _ => {}
    }
    panic!("Unhandled encoded value: {}", string::String::from_utf8_lossy(encoded_value))
}
