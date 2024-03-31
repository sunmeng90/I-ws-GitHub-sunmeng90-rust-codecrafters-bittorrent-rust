use std::{env, string};

use itertools::Itertools;
use sha1::Digest;

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
            let decoded = decode(&encoded_content).0;
            let json_value = serde_json::Value::try_from(decoded).unwrap();
            let torrent: bencode::Torrent = serde_json::from_value(json_value).unwrap();
            // let torrent: bencode::Torrent = serde_bencode::from_bytes(&encoded_content).unwrap();
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
            // println!("Encode: length: {:?}, {:?}", encoded_info.len(), encoded_info)
            println!("Piece Length: {:?}", torrent.info.piece_length);
            println!("Piece Hashes:");
            //  e876f67a2a8886e8f36b136726c30fa29703022d
            //  6e2275e604a0766656736e81ff10b55204ad8d35
            //  f00d937a0213df1982bc8d097227ad9e909acc17
            torrent.info.pieces.0.iter().for_each(|x| {
                println!("{:}", hex::encode(x));
            });
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}


