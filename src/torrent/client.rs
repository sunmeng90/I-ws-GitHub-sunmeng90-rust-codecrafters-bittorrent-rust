use std::net::{Ipv4Addr, SocketAddrV4};
use std::process;

use anyhow::{anyhow, Context, Result};
use bytes::{Buf, BufMut, BytesMut};
// has to import explicitly
use futures_util::SinkExt;
// has to import explicitly
use futures_util::StreamExt;
use serde_bencode::from_bytes;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio_util::codec::{Decoder, Encoder, Framed};

use crate::torrent::exchange::{BlockReqPayload, ExchangeMsg, MsgType};
use crate::torrent::handeshake::Handshake;
use crate::torrent::serde::peers::Peer;
use crate::torrent::torrent::{Keys, PeersResponse, Torrent};
use crate::{get_file_length, url_encode};

const BLOCK_MAX: usize = 1 << 14;
const MAX: usize = 1 << 16;

pub struct Client {
    pub torrent: Torrent,
    pub c: reqwest::Client,
    peer_conn: Option<TcpStream>,
}

impl Client {
    pub fn new(torrent: Torrent) -> Self {
        Self {
            torrent,
            c: reqwest::Client::new(),
            peer_conn: None,
        }
    }

    pub async fn get_peers(&self) -> Result<Vec<Peer>> {
        let url = format!(
            "{}?info_hash={}",
            self.torrent.announce,
            url_encode(self.torrent.info_hash().as_slice())
        );
        let builder = self.c.get(url).query(&[
            ("peer_id", "00112233445566778899".parse().unwrap()),
            ("port", 6881.to_string()),
            ("uploaded", "0".parse().unwrap()),
            ("downloaded", "0".parse().unwrap()),
            ("left", get_file_length(&self.torrent).to_string()),
            ("compact", "1".parse().unwrap()),
        ]);
        let req = builder.build();
        let resp = self.c.execute(req.unwrap()).await?;

        let bytes = resp.bytes().await?;
        println!("Resp: {:?}", String::from_utf8_lossy(&bytes));
        let peers_resp: PeersResponse = from_bytes::<PeersResponse>(&bytes).unwrap();
        Ok(peers_resp.peers)
    }

    pub async fn handshake(&mut self) -> Result<Handshake> {
        let peers = self.get_peers().await?;
        let peer = peers.iter().next().unwrap();
        let peer = SocketAddrV4::new(
            peer.0.parse::<Ipv4Addr>().context("parse peer ip").unwrap(),
            peer.1,
        );
        println!("connecting to peer {:?}", peers);
        let peer_conn = TcpStream::connect(peer)
            .await
            .context("connect to peer")
            .unwrap();
        self.peer_conn = Some(peer_conn);

        let mut handshake =
            Handshake::new(self.torrent.info_hash().into(), *b"00112233445566778899");
        {
            const SIZE: usize = std::mem::size_of::<Handshake>();
            // This line casts a mutable reference to handshake to a mutable pointer to an array of bytes of the same SIZE as Handshake.
            let handshake_bytes = &mut handshake as *mut Handshake as *mut [u8; SIZE];
            // Safety: Handshake is a POD with repr(c)
            // This block contains unsafe code that dereferences the pointer created in the
            // previous line to obtain a mutable reference to an array of bytes.
            let handshake_bytes: &mut [u8; SIZE] = unsafe { &mut *handshake_bytes };
            println!("handshake start");
            // Option.unwrap will move value, so instead we get a mut ref to connection
            if let Some(conn) = self.peer_conn.as_mut() {
                conn.write_all(handshake_bytes)
                    .await
                    .context("write handshake")
                    .unwrap();
                println!("handshake read resp");
                conn.read_exact(handshake_bytes)
                    .await
                    .context("read handshake")
                    .unwrap();
                anyhow::ensure!(handshake.length == 19);
                anyhow::ensure!(&handshake.bittorrent == b"BitTorrent protocol");
                return Ok(handshake);
            }
        }
        Err(anyhow!("handshake failed, non peer connection"))
    }

    pub async fn download_pieces(&mut self, piece_idx: usize) -> Result<()> {
        let handshake = self.handshake().await?;

        println!("handshake peer: {:?}", hex::encode(handshake.peer_id));
        let conn = self.peer_conn.as_mut().unwrap();
        
        println!("Wait for BitField msg");
        let msg = ExchangeMsg::read_from(conn).await?;
        anyhow::ensure!(msg.message_id.unwrap() == MsgType::BitField);
        
        println!("Send Interested msg");
        let msg = ExchangeMsg::new(MsgType::Interested, Vec::new());
        let mut peer = Framed::new(conn, MessageCodec);
        peer.send(msg).await.context("send interested message")?;
        
        println!("Wait for Unchoke msg");
        let msg = peer.next().await.context("invalid unchoke msg")?;
        assert_eq!(MsgType::Unchoke, msg.unwrap().message_id.unwrap());

        // 1. download piece with index piece_idx
        let info = &self.torrent.info;
        println!("Download piece {:?}", info.pieces.0);
        let piece_hash = info.pieces.0[piece_idx];
        let piece_count = info.pieces.0.len();
        let req_piece_size = if piece_idx == piece_count - 1 {
            // if last piece
            let total_length = match info.keys {
                Keys::Single { length } => length,
                _ => 0,
            };
            let md = total_length % info.piece_length;

            if md == 0 {
                info.piece_length
            } else {
                md
            }
        } else {
            info.piece_length
        };
        // 2. create request for each block of the piece
        // each block is identified:
        // index: piece index
        // begin: offset within the piece
        // length: length of the block
        
        let mut piece_buf:Vec<u8> = Vec::with_capacity(req_piece_size);
        // 2.1 calc offset for each block
        let blocks_count = (req_piece_size + BLOCK_MAX - 1) / BLOCK_MAX;
        for b in 0..blocks_count {
            let block_size = if b == blocks_count - 1 {
                let md = req_piece_size % BLOCK_MAX;
                if md != 0 {
                    md
                } else {
                    BLOCK_MAX
                }
            } else {
                BLOCK_MAX
            };

            let mut block_payload =
                BlockReqPayload::new(piece_idx as u32, (b * BLOCK_MAX) as u32, block_size as u32);
            let block_payload = Vec::from(block_payload.as_bytes_mut());
            let block_req = ExchangeMsg::new(MsgType::Request, block_payload);
            println!("Download block {} / {}", b, blocks_count);

            peer.send(block_req)
                .await
                .with_context(|| format!("send request for block {b}"))?;

            let piece = peer
                .next()
                .await
                .expect("peer always send a piece")
                .with_context(|| "peer message is invalid")?;

            assert_eq!(piece.message_id, Some(MsgType::Piece));
            assert!(!piece.payload.is_empty());
            // println!("block {}, resp payload: {:?}", b, piece.payload.clone());
            println!("Download receive block size: {}", piece.payload.len());
            // accumulate block resp
            println!("pre append buf size: {}", &piece_buf.len());
            piece_buf.append(&mut piece.payload.clone());

        }
        println!("piece len: {}", &piece_buf.len());
        std::fs::write(format!("piece_0_{}", process::id()), String::from_utf8(piece_buf)?).unwrap();

        // send request
        Ok(())
    }
}

pub struct MessageCodec;

impl Encoder<ExchangeMsg> for MessageCodec {
    type Error = anyhow::Error;

    fn encode(&mut self, item: ExchangeMsg, dst: &mut BytesMut) -> Result<()> {
        let len_bytes = u32::to_be_bytes((item.payload.len() + 1) as u32);
        dst.reserve(4 + 1 + item.payload.len());
        dst.extend_from_slice(&len_bytes);
        dst.put_u8(item.message_id.unwrap() as u8);
        dst.extend_from_slice(&item.payload);
        Ok(())
    }
}

impl Decoder for MessageCodec {
    type Item = ExchangeMsg;
    type Error = anyhow::Error;

    fn decode(&mut self, src: &mut BytesMut) -> anyhow::Result<Option<Self::Item>> {
        if src.len() < 4 {
            return Ok(None);
        }

        let mut len_bytes = [0u8; 4];
        len_bytes.copy_from_slice(&src[..4]);

        // len is the total bytes of messageId + Payload
        let len: usize = u32::from_be_bytes(len_bytes) as usize;
        println!("expect msg len: {:?}", len);
        if len == 0 {
            // discard heartbeat msg
            src.advance(4);
            // and then try again in case the buffer has more messages
            return self.decode(src);
        }
        // println!("Src len: {:?}", src.len());
        if src.len() < 5 {
            // Not enough data to read tag marker.
            return Ok(None);
        }

        // Check that the length is not too large to avoid a denial of
        // service attack where the server runs out of memory.
        if len > MAX {
            println!("Decode msg: Frame of length {:?} is too large.", len);
            // return Err(std::io::Error::new(
            //     std::io::ErrorKind::InvalidData,
            //     format!("Frame of length {} is too large.", len),
            // ));
            return Ok(None); // TODO
        }

        // if not all data arrived, we need to wait
        if src.len() < 4 + len {
            // The full string has not yet arrived.
            //
            // We reserve more space in the buffer. This is not strictly
            // necessary, but is a good idea performance-wise.
            src.reserve(4 + len - src.len());
            // We inform the Framed that we need more bytes to form the next
            // frame.
            return Ok(None);
        }

        let msg_type = src[4].try_into()?;
        // src.advance(1);
        let data = if src.len() > 5 {
            src[5..len + 4].to_vec()
        }else{
            Vec::new()
        };
        let actual_len = &data.len();
        let msg = ExchangeMsg::new(msg_type, data);
        src.advance(4 + len);
        println!("receive msg len: {:?}", actual_len);
        Ok(Some(msg))
    }
}

// pub async fn read_from<R>(reader: &mut R) -> Result<Self>
// where
//     R: AsyncRead + Unpin,
// {
//     let len = reader.read_u32().await?;
//     println!("resp len: {}", len);
//     if len > 0 {
//         return match reader.read_u8().await?.try_into()? {
//             MsgType::BitField => {
//                 let mut buf = Vec::new();
//                 reader.read_to_end(&mut buf).await?;
//                 println!("message payload: {:?}", buf);
//                 Ok(ExchangeMsg {
//                     len_prefix: len,
//                     message_id: Some(MsgType::BitField),
//                     payload: buf,
//                 })
//             }
//             _ => Err(anyhow!("")),
//         };
//     }
//
//     Err(anyhow!(""))
// }
