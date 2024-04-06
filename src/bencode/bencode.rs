use serde::Serialize;
use std::collections::BTreeMap;

#[derive(Debug, Serialize)]
pub enum Bencode {
    Byte(Vec<u8>),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<String, Bencode>),
}
