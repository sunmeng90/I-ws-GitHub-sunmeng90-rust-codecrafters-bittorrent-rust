use std::collections::BTreeMap;
use std::string;

use serde::{Deserialize, Serialize};
use serde_with::serde_as;

use crate::bencode::hashes::Hashes;

pub mod decode;

#[derive(Debug, Serialize)]
pub enum Bencode {
    Byte(Vec<u8>),
    Integer(i64),
    List(Vec<Bencode>),
    Dict(BTreeMap<string::String, Bencode>),
}


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

pub mod bytes_or_string {
    use std::{cmp, fmt};

    use serde::de::{SeqAccess, Visitor};
    use serde::Deserializer;

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

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
            // decoded json array(Vec<u8>) to string
            let len = cmp::min(seq.size_hint().unwrap_or(0), 4096);
            let mut bytes = Vec::with_capacity(len);

            while let Some(b) = seq.next_element()? {
                bytes.push(b);
            }

            Ok(String::from_utf8(bytes).unwrap())
        }
    }
}

mod hashes {
    use std::fmt::{Formatter, Write};

    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use serde::de::{Error, SeqAccess, Visitor};

    #[derive(Debug)]
    pub struct Hashes(pub Vec<[u8; 20]>);

    struct HashesVisitor;

    impl<'de> Visitor<'de> for HashesVisitor {
        type Value = Hashes;

        fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
            formatter.write_str("a byte string whose length is a multiple of 20")
        }

        fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E> where E: Error {
            if v.len() % 20 != 0 {
                return Err(E::custom(format!("length is {}", v.len())));
            }
            let hashes = Hashes(v.chunks_exact(20)
                .map(|chunks_20| chunks_20.try_into().expect("guaranteed to be length of 20"))
                .collect());
            Ok(hashes)
        }

        fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error> where A: SeqAccess<'de> {
            let mut v = Vec::with_capacity(seq.size_hint().unwrap());

            while let Some(b) = seq.next_element()? {
                v.push(b);
            }

            self.visit_bytes(&v)
        }
    }

    impl<'de> Deserialize<'de> for Hashes {
        fn deserialize<D>(deserializer: D) -> Result<Self, D::Error> where D: Deserializer<'de> {
            deserializer.deserialize_bytes(HashesVisitor)
        }
    }

    impl Serialize for Hashes {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
            let concated_hashes = self.0.concat();
            serializer.serialize_bytes(&concated_hashes)
        }
    }
}
