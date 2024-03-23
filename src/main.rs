use std::{env, string};
use std::str::FromStr;

use ::serde::Deserializer;
use sha1::Digest;
use sha1::digest::Update;

use crate::bencode::decode::decode;
use crate::bencode::Keys::{Multiple, Single};

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
            // let decoded = decode(&encoded_content).0;
            // let json_value = serde_json::Value::try_from(decoded).unwrap();
            // let torrent: bencode::Torrent = serde_json::from_value(json_value).unwrap();
            let torrent   : bencode::Torrent= serde_bencode::from_bytes(&encoded_content).unwrap();
            let encoded_info = serde_bencode::to_bytes(&(torrent.info)).unwrap();
            let mut hasher = sha1::Sha1::new();
            Digest::update(&mut hasher, encoded_info.clone());
            let hash = hasher.finalize();
            println!("Tracker URL: {:?}", torrent.announce);
            let len = match torrent.info.keys {
                Single { length } => length,
                Multiple { .. } => 0
            };
            println!("Length: {:?}", len);
            println!("Info Hash: {:x}", hash);
            println!("Encode: {:?}", encoded_info)
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}


