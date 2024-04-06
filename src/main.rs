use anyhow::Context;
use std::net::SocketAddrV4;
use std::{env, string};

use sha1::Digest;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::bencode::decode::decode;
use crate::torrent::handeshake::Handshake;
use crate::torrent::torrent::Keys::{Multiple, Single};
use crate::torrent::torrent::PeersResponse;
use crate::torrent::torrent::Torrent;
mod bencode;
mod torrent;

// Available if you need it!

// Usage: your_bittorrent.sh decode "<encoded_value>"
#[tokio::main]
async fn main() -> anyhow::Result<()> {
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
            let torrent: Torrent = serde_json::from_value(json_value).unwrap();
            let hash = calc_info_hash(&torrent);
            println!("Tracker URL: {:?}", torrent.announce);
            println!("Length: {:?}", get_file_length(&torrent));
            println!("Info Hash: {:?}", hex::encode(hash));
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
            let client = reqwest::Client::new();
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
                ("left", get_file_length(&torrent).to_string()),
                ("compact", "1".parse().unwrap()),
            ]);
            let req = builder.build();
            let resp = client.execute(req.unwrap()).await.unwrap();

            let bytes = resp.bytes().await.unwrap();
            println!("Resp: {:?}", String::from_utf8_lossy(&bytes));
            let peers_resp = serde_bencode::from_bytes::<PeersResponse>(&bytes).unwrap();
            peers_resp
                .peers
                .iter()
                .for_each(|peer| println!("{}:{}", peer.0, peer.1));
        }
        "handshake" => {
            let peer = &args[3];
            // 165.232.33.77:51467
            // 178.62.85.20:51489
            // 178.62.82.89:51448

            // println!("handshake with {}", peer);
            let encoded_content = std::fs::read(&args[2]).unwrap();
            let torrent = serde_bencode::from_bytes::<Torrent>(&encoded_content).unwrap();
            let hash = calc_info_hash(&torrent);

            let peer = peer
                .parse::<SocketAddrV4>()
                .context("parse peer address")
                .unwrap();
            let mut peer = tokio::net::TcpStream::connect(peer)
                .await
                .context("connect to peer")
                .unwrap();

            let mut handshake = Handshake::new(hash.into(), *b"00112233445566778123");
            {
                const SIZE: usize = std::mem::size_of::<Handshake>();
                // This line casts a mutable reference to handshake to a mutable pointer to an array of bytes of the same SIZE as Handshake.
                let handshake_bytes = &mut handshake as *mut Handshake as *mut [u8; SIZE];
                // Safety: Handshake is a POD with repr(c)
                // This block contains unsafe code that dereferences the pointer created in the
                // previous line to obtain a mutable reference to an array of bytes.
                let handshake_bytes: &mut [u8; SIZE] = unsafe { &mut *handshake_bytes };

                peer.write_all(handshake_bytes)
                    .await
                    .context("write handshake")
                    .unwrap();

                peer.read_exact(handshake_bytes)
                    .await
                    .context("read handshake")
                    .unwrap();
            }
            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");
            println!("Peer ID: {:}", hex::encode(&handshake.peer_id) );

        }
        _ => {
            println!("unknown command: {}", args[1]);
        }
    };

    Ok(())
}

fn calc_info_hash(torrent: &Torrent) -> [u8; 20] {
    let encoded_info = serde_bencode::to_bytes(&(torrent.info)).unwrap();
    let mut hasher = sha1::Sha1::new();
    Digest::update(&mut hasher, encoded_info.clone());
    let hash = hasher.finalize();
    hash.try_into().unwrap()
}

fn get_file_length(torrent: &Torrent) -> usize {
    match torrent.info.keys {
        Single { length } => length,
        Multiple { .. } => 0,
    }
}

fn url_encode(bytes: &[u8]) -> String {
    let _result = String::new();
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
