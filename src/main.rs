use crate::bencode::decode::decode;
use crate::bencode::Keys::{Multiple, Single};
use form_urlencoded::byte_serialize;
use http::StatusCode;
use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use std::fmt::format;
use std::io::Read;
use std::{env, string};
use url::form_urlencoded;

use crate::bencode::{PeersResponse, Torrent};
use sha1::digest::Output;
use sha1::Digest;

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
            let decoded_val: serde_json::Value =
                decode(encoded_value.as_bytes()).0.try_into().unwrap();
            println!("{}", serde_json::to_string(&decoded_val).unwrap());
        }
        "info" => {
            let encoded_content = std::fs::read(&args[2]).unwrap();
            let decoded = decode(&encoded_content).0;
            let json_value = serde_json::Value::try_from(decoded).unwrap();
            let torrent: bencode::Torrent = serde_json::from_value(json_value).unwrap();
            let hash = calc_info_hash(&torrent);
            println!("Tracker URL: {:?}", torrent.announce);
            println!("Length: {:?}", get_file_length(&torrent));
            println!("Info Hash: {:?}", hex::encode(&hash));
            println!("Piece Length: {:?}", torrent.info.piece_length);
            println!("Piece Hashes:");
            //  e876f67a2a8886e8f36b136726c30fa29703022d
            //  6e2275e604a0766656736e81ff10b55204ad8d35
            //  f00d937a0213df1982bc8d097227ad9e909acc17
            torrent.info.pieces.0.iter().for_each(|x| {
                println!("{:}", hex::encode(x));
            });
        }
        "peers" => {
            let encoded_content = std::fs::read(&args[2]).unwrap();
            let torrent = serde_bencode::from_bytes::<Torrent>(&encoded_content).unwrap();
            let hash = calc_info_hash(&torrent);
            let client = reqwest::blocking::Client::new();
            let url = format!(
                "{}?info_hash={}",
                torrent.announce,
                url_encode(hash.as_slice())
            );
            let builder = client.get(url).query(&[
                ("peer_id", "00112233445566778899".parse().unwrap()),
                ("port", 6881.to_string()),
                ("uploaded", "0".parse().unwrap()),
                ("downloaded", "0".parse().unwrap()),
                ("left",  get_file_length(&torrent).to_string()),
                ("compact", "1".parse().unwrap()),
            ]);
            let req = builder.build();
            let resp = client.execute(req.unwrap()).unwrap();
            let peers_resp = serde_bencode::from_bytes::<PeersResponse>(&resp.bytes().unwrap()).unwrap();
            peers_resp.peers.iter().for_each(|peer|{
                println!("{}:{}", peer.0, peer.1)
            });
           
        }
        _ => {
            println!("unknown command: {}", args[1])
        }
    }
}

fn calc_info_hash(torrent: &Torrent) -> Vec<u8> {
    let encoded_info = serde_bencode::to_bytes(&(torrent.info)).unwrap();
    let mut hasher = sha1::Sha1::new();
    Digest::update(&mut hasher, encoded_info.clone());
    let hash = hasher.finalize();
    hash.as_slice().to_vec()
}

fn get_file_length(torrent: &Torrent) -> usize {
    let len = match torrent.info.keys {
        Single { length } => length,
        Multiple { .. } => 0,
    };
    len
}

fn url_encode(bytes: &[u8]) -> String {
    let result = String::new();
    bytes
        .iter()
        .map(|&b| match b {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'.' | b'_' | b'~' => {
                String::from(b as char)
            }
            _ => format!("%{:02X}", b),
        })
        .collect()
}
