use crate::torrent::serde::peers;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use crate::torrent::serde::hashes::Hashes;
use crate::torrent::serde::peers::Peer;
use crate::torrent::serde::bytes_or_string;
#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Torrent {
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub announce: String,
    #[serde(rename = "created by")]
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub created_by: String,
    pub info: Info,
}

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: Hashes,
    #[serde(flatten)]
    pub keys: Keys,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
pub enum Keys {
    Single { length: usize },
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
    pub  interval: usize,
    #[serde(rename = "min interval")]
    pub  min_interval: usize,
    #[serde(deserialize_with = "peers::deserialize_vec")]
    pub  peers: Vec<Peer>,
}
