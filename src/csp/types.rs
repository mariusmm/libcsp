pub struct CspPacket {
    pub frame_begin: [u8; 4],
    pub length: usize,
    pub id: CspId,
    pub data :  [u8; 256], 
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

    pub fn length(mut self, length: usize) -> Self {
        self.length = length;
        self
    }

    pub fn id(mut self, id: CspId) -> Self {
        self.id = id;
        self
    }

    pub fn data(mut self, data: [u8; 256]) -> Self {
        self.data = data;
        self
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

    pub fn pri (mut self, pri: u8) -> Self {
        self.pri = pri;
        self
    }

    pub fn flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    pub fn src(mut self, src:  u16) -> Self {
        self.src = src;
        self
    }

    pub fn dst(mut self, dst: u16) -> Self {
        self.dst = dst;
        self
    }

    pub fn dport(mut self, dport: u8) -> Self {
        self.dport = dport;
        self
    }

    pub fn sport(mut self, sport: u8) -> Self {
        self.sport = sport;
        self
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csppacket_test() {
        let test = CspPacket::new();
        assert_eq!(test.frame_begin, [0u8; 4]);
        assert_eq!(test.length, 0);
        assert_eq!(test.data, [0u8; 256]);
    }

    #[test]
    fn cspid_test() {
       let test = CspId::new().flags(5).pri(2).dport(23).sport(37).src(125).dst(90);
       assert_eq!(test.pri, 2);
       assert_eq!(test.flags, 5);
       assert_eq!(test.src, 125);
       assert_eq!(test.dst, 90);
       assert_eq!(test.dport, 23);
       assert_eq!(test.sport, 37);
    }
}