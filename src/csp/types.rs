pub struct CspPacket {
    pub frame_begin: [u8; 4],
    pub length: u16,
    pub id: CspId,
    pub data: [u8; 256],
}

#[derive(Clone, Copy)]
pub struct CspId {
    pub pri: u8,
    pub flags: u8,
    pub src: u16,
    pub dst: u16,
    pub dport: u8,
    pub sport: u8,
}

#[allow(dead_code)]
pub enum CspError {
    CspNoError,
    CspError
}

impl CspPacket {
    pub fn new () -> Self {
        Self {frame_begin: [0; 4],
            length: 0,
            id: CspId::new(),
            data: [0; 256],
        }
    }
}

impl CspId {
    pub fn new () -> Self {
        Self {
            pri: 0,
            flags: 0,
            src: 0,
            dst: 0,
            dport: 0,
            sport: 0,
        }
    }
}