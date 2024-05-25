use anyhow::{anyhow, Result};
use bytes::buf::Reader;
use serde::Deserialize;
use tokio::io::{AsyncRead, AsyncReadExt};

#[derive(Deserialize, Debug)]
pub struct ExchangeMsg {
    pub len_prefix: u32,
    pub message_id: Option<MsgType>,
    #[serde(default)]
    pub payload: Vec<u8>,
}

#[derive(Deserialize, Debug, PartialEq)]
#[repr(u8)]
pub enum MsgType {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    BitField = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl TryFrom<u8> for MsgType {
    type Error = anyhow::Error;
    fn try_from(value: u8) -> Result<Self> {
        match value {
            0 => Ok(Self::Choke),
            1 => Ok(Self::Unchoke),
            2 => Ok(Self::Interested),
            3 => Ok(Self::NotInterested),
            4 => Ok(Self::Have),
            5 => Ok(Self::BitField),
            6 => Ok(Self::Request),
            7 => Ok(Self::Piece),
            8 => Ok(Self::Cancel),
            v => Err(anyhow!("invalid message type {}", v).context("parse message type")),
        }
    }
}

impl ExchangeMsg {
    pub fn new(t: MsgType, payload: Vec<u8>) -> Self {
        ExchangeMsg {
            len_prefix: payload.len() as u32,
            message_id: Some(t),
            payload,
        }
    }
    pub async fn read_from<R>(reader: &mut R) -> Result<Self>
    where
        R: AsyncRead + Unpin,
    {
        let len = reader.read_u32().await?;
        // println!("resp len: {}", len);
        if len > 0 {
            return match reader.read_u8().await?.try_into()? {
                MsgType::BitField => {
                    let mut buf = Vec::with_capacity((len -1) as usize); // TODO: Vec::new ?
                    reader.read_buf(&mut buf).await?;  // read and advance
                    println!("message payload: {:?}", buf);
                    Ok(ExchangeMsg {
                        len_prefix: len,
                        message_id: Some(MsgType::BitField),
                        payload: buf,
                    })
                }
                _ => {
                    Err(anyhow!("failed to read"))
                },
            };
        }

        Err(anyhow!(""))
    }
}

#[derive(Debug)]
#[repr(C)]
pub struct BlockReqPayload {
    pub index: [u8; 4], // piece index
    pub begin: [u8; 4],
    pub length: [u8; 4],
}

impl BlockReqPayload {
    pub fn new(index: u32, begin: u32, length: u32) -> Self {
        BlockReqPayload {
            index: index.to_be_bytes(),
            begin: begin.to_be_bytes(),
            length: length.to_be_bytes(),
        }
    }

    pub fn as_bytes_mut(&mut self) -> &mut [u8] {
        let bytes = self as *mut Self as *mut [u8; std::mem::size_of::<Self>()];
        // Safety: Handshake is a POD with repr(c)
        let bytes: &mut [u8; std::mem::size_of::<Self>()] = unsafe { &mut *bytes };
        bytes
    }
}