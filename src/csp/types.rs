pub struct CspPacket {
    pub frame_begin: [u8; 4],
    pub length: u16,
    pub id: CspId,
    pub data: [u8; 256],
}

pub struct CspId {
    pub pri: u8,
    pub flags: u8,
    pub src: u16,
    pub dst: u16,
    pub dport: u8,
    pub sport: u8,
}
