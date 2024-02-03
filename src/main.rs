mod bencode;

use std::{env, string};
use std::collections::BTreeMap;

use serde::Serialize;
use crate::bencode::decode::decode;

// Available if you need it!

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<string::String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_val: serde_json::Value = decode(encoded_value).0.try_into().unwrap();
        println!("{}", serde_json::to_string(&decoded_val).unwrap());
    } else {
        println!("unknown command: {}", args[1])
    }
}
