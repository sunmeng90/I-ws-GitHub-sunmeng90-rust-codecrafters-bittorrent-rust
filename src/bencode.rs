use std::collections::BTreeMap;
use std::string;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

pub mod decode;

#[derive(Debug, Serialize)]
pub enum Bencode {
    String(string::String),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<string::String, Bencode>),
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Torrent {
    pub announce: String,
    #[serde(rename = "created by")]
    pub created_by: String,
    pub info: Info,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: ByteBuf,
    #[serde(flatten)]
    pub keys: Keys,
}


#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Keys {
    Single {
        length: usize,
    },
    Multiple {
        files: Vec<FileInfo>
    },
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileInfo {
    length: usize,
    path: Vec<String>,
}
