#[repr(C)]
pub struct Handshake {
    pub length: u8,
    pub protocol: [u8; 19],
    pub zero: [u8; 8],
    pub info_hash: [u8; 20],
    pub peer_id: [u8; 20],
}

impl Handshake {
    pub fn new(info_hash: [u8; 20], peer_id: [u8; 20]) -> Self {
        Self {
            length: 19,
            protocol: *b"Bittorrent protocol",
            zero: [0; 8],
            info_hash,
            peer_id,
        }
    }
}
