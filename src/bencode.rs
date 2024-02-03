use std::collections::BTreeMap;
use std::string;
use serde::Serialize;

pub mod decode;


#[derive(Debug, Serialize)]
pub enum Bencode {
    String(string::String),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<string::String, Bencode>),
}
