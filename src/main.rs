use std::{env, string};

use crate::bencode::decode::decode;

mod bencode;
mod serde;

// Available if you need it!

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<string::String> = env::args().collect();
    let command = &args[1];
    match command.as_str() {
        "decode" => {
            let encoded_value = &args[2];
            let decoded_val: serde_json::Value = decode(encoded_value.as_bytes()).0.try_into().unwrap();
            println!("{}", serde_json::to_string(&decoded_val).unwrap());
        }
        "info" => {
            let encoded_content = std::fs::read(&args[2]).unwrap();
            let decoded = decode(&encoded_content).0;
            let json_value = serde_json::Value::try_from(decoded).unwrap();
            let announce = json_value.get("announce").unwrap();
            let info = json_value.get("info").unwrap();
            println!("Tracker URL: {:?}\nLength: {:?}", announce.as_str().unwrap(), info.get("length").unwrap().as_i64().unwrap());
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}

