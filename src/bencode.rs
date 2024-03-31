use serde_with::FromInto;
use serde_with::TryFromInto;
use serde_with::BytesOrString;
use std::collections::BTreeMap;
use std::string;
use clap::builder::Str;
use serde_with::DisplayFromStr;
use serde::{Deserialize, Serialize, Serializer};
use serde_bytes::ByteBuf;
use serde_with::serde_as;

pub mod decode;

#[derive(Debug, Serialize)]
pub enum Bencode {
    Byte(Vec<u8>),
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

#[serde_as]
#[derive(Serialize, Deserialize, Debug)]
pub struct Info {
    #[serde(deserialize_with = "bytes_or_string::deserialize")]
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: usize,
    pub pieces: ByteBuf,
    #[serde(flatten)]
    pub keys: Keys,
}

pub mod bytes_or_string {
    use std::fmt;
    use std::fmt::Write;
    use serde::de::{SeqAccess, Visitor};
    use serde::Deserializer;
    use super::*;

    /// Deserialize a String from either bytes or string
    pub fn deserialize<'de, D>(deserializer: D) -> Result<String, D::Error>
        where
            D: Deserializer<'de>,
    {
        deserializer.deserialize_any(BytesOrStringVisitor)
    }

    struct BytesOrStringVisitor;

    impl<'de> Visitor<'de> for BytesOrStringVisitor {
        type Value = String;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("a list of bytes or a string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E> {
            Ok(v.to_string())
        }

        fn visit_string<E>(self, v: String) -> Result<Self::Value, E> {
            Ok(v)
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> {
            Ok(String::from_utf8_lossy(v).parse().unwrap())
        }

        fn visit_byte_buf<E>(self, v: Vec<u8>) -> Result<Self::Value, E> {
            Ok(String::from_utf8(v).unwrap())
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
        {
            let mut res = Vec::with_capacity(seq.size_hint().unwrap_or(0));
            while let Some(value) = seq.next_element()? {
                res.push(value);
            }
            Ok(String::from_utf8(res).unwrap())
        }
    }
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



