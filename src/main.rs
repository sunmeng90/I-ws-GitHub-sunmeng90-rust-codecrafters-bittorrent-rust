use std::net::SocketAddrV4;
use std::{env, string};

use anyhow::Context;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use torrent::client::Client;
use torrent::handeshake::Handshake;
use torrent::torrent::Keys::{Multiple, Single};
use torrent::torrent::Torrent;

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
            println!("{}", bencode::decode::decode_str(&args[2]));
        }
        "info" => {
            let torrent: Torrent = Torrent::from_file_old(&args[2]);
            println!("{}", torrent.format_info())
        }
        "peers" => {
            let torrent = Torrent::from_file(&args[2]);
            Client::new(torrent)
                .get_peers()
                .await
                .unwrap()
                .iter()
                .for_each(|peer| println!("{}:{}", peer.0, peer.1));
        }
        "handshake" => {
            let peer = &args[3];
            // 165.232.33.77:51467
            // 178.62.85.20:51489
            // 178.62.82.89:51448

            // println!("handshake with {}", peer);
            let torrent = Torrent::from_file(&args[2]);
            let handshake = Client::new(torrent).handshake().await.unwrap();
            assert_eq!(handshake.length, 19);
            assert_eq!(&handshake.bittorrent, b"BitTorrent protocol");
            println!("Peer ID: {:}", hex::encode(&handshake.peer_id));
        }
        "download_piece" => {
            // args.iter().into_iter().for_each(|a| println!("{}", a));
            let torrent = Torrent::from_file(&args[4]);
            let piece: usize = args[5].parse()?;
            Client::new(torrent).download_pieces(piece).await?
        }
        _ => {
            println!("unknown command: {}", args[1]);
        }
    };

    Ok(())
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
