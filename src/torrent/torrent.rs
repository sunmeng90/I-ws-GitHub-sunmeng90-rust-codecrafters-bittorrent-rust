use anyhow::Context;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_bencode::from_bytes;
use serde_with::serde_as;
use sha1::Digest;

use crate::bencode::decode::decode;
use crate::torrent::serde::bytes_or_string;
use crate::torrent::serde::hashes::Hashes;
use crate::torrent::serde::peers;
use crate::torrent::serde::peers::Peer;
use crate::torrent::torrent::Keys::{Multiple, Single};

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Torrent {
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub announce: String,
    #[serde(rename = "created by")]
    #[serde(default)] // if no value, then use String::default
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub created_by: String,
    pub info: Info,
}

impl Torrent {
    pub fn info_hash(&self) -> [u8; 20] {
        let encoded_info = serde_bencode::to_bytes(&(self.info)).unwrap();
        let mut hasher = sha1::Sha1::new();
        Digest::update(&mut hasher, encoded_info.clone());
        let hash = hasher.finalize();
        hash.try_into().unwrap()
    }

    pub fn from_file_old(file_path: &str) -> Self {
        let encoded_content = std::fs::read(file_path).unwrap();
        let decoded = decode(&encoded_content).0;
        let json_value = serde_json::Value::try_from(decoded).unwrap();
        serde_json::from_value(json_value).unwrap()
    }

    pub fn from_file(file_path: &str) -> Self {
        let encoded_content = std::fs::read(file_path)
            .context(format!("can not read file {}", file_path))
            .unwrap();
        from_bytes::<Torrent>(&encoded_content).unwrap()
    }

    fn get_file_length(&self) -> usize {
        match self.info.keys {
            Single { length } => length,
            Multiple { .. } => 0,
        }
    }

    pub fn format_info(&self) -> String {
        format!(
            r#"Tracker URL: {}
Length: {}
Info Hash: {}
Piece Length: {}
Piece Hashes:
{}"#,
            self.announce,
            self.get_file_length(),
            hex::encode(self.info_hash()),
            self.info.piece_length,
            self.info
                .pieces
                .0
                .iter()
                .format_with("\n", |e, f| { f(&format_args!("{:}", hex::encode(e))) })
        )
        //  e876f67a2a8886e8f36b136726c30fa29703022d
        //  6e2275e604a0766656736e81ff10b55204ad8d35
        //  f00d937a0213df1982bc8d097227ad9e909acc17
    }
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,  // byte size for each piece
    pub pieces: Hashes,
    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Keys {
    Single { length: usize },  // total bytes for file
    Multiple { files: Vec<FileInfo> },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    length: usize,
    path: Vec<String>,
}

/// peers

#[serde_as]
#[derive(Deserialize, Debug)]
pub struct PeersResponse {
    pub complete: usize,
    pub incomplete: usize,
    pub interval: usize,
    #[serde(default)]
    #[serde(rename = "min interval")]
    pub min_interval: usize,
    #[serde(deserialize_with = "peers::deserialize_vec")]
    pub peers: Vec<Peer>,
}
