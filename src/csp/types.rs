// SPDX-License-Identifier: MIT
use std::io;

use crc::{Crc, CRC_32_ISCSI};

use crate::csp::interface::*;

pub const CSPCRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

pub fn csp_send_direct_iface<Intf>(
    _idout: &CspId,
    packet: &mut CspPacket,
    iface: &mut Intf,
    via: u16,
    from_me: bool,
) -> Result<(), io::Error>
where
    Intf: NextHop,
{
    return iface.next_hop(via, packet, from_me);
}

#[derive(Clone)]
pub struct CspPacket {
    pub frame_begin: [u8; 4],
    pub id: CspId,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy)]
pub struct CspId {
    pub pri: u8,
    pub flags: u8,
    pub src: u8,
    pub dst: u8,
    pub dport: u8,
    pub sport: u8,
}

#[derive(PartialEq)]
pub enum ConnState {
    ConnOpen,
    ConnClosed,
}

pub struct CspConnection {
    pub opts: u32,
    pub state: ConnState,
    pub idout: CspId,
}

pub struct CspFIFO {
    pub iface: CspIface,
    pub packet: CspPacket,
}

#[allow(dead_code)]
pub enum CspError {
    CspNoError,
    CspError,
}

pub enum CspServices {
    CspCMP = 0,
    CspPing = 1,
    CspPs = 2,
    CspMemFree = 3,
    CspReboot = 4,
    CspBufFree = 5,
    CspUptime = 6,
}

pub enum CspPriorities {
    CspPrioCritical,
    CspPrioHigh,
    CspPrioNormal,
    CspPrioLow,
}

impl CspPacket {
    pub fn new() -> Self {
        Self {
            frame_begin: [0; 4],
            id: CspId::new(),
            data: Vec::new(),
        }
    }

    pub fn id(mut self, id: CspId) -> Self {
        self.id = id;
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn csp_crc32_append(&mut self) {
        let calc_crc = CSPCRC32.checksum(&mut self.data);

        self.data.push(((calc_crc & 0xFF000000) >> 24) as u8);
        self.data.push(((calc_crc & 0x00FF0000) >> 16) as u8);
        self.data.push(((calc_crc & 0x0000FF00) >> 8) as u8);
        self.data.push((calc_crc & 0x000000FF) as u8);
    }
}

impl CspId {
    pub fn new() -> Self {
        Self {
            pri: 0,
            flags: 0,
            src: 0,
            dst: 0,
            dport: 0,
            sport: 0,
        }
    }

    pub fn pri(mut self, pri: u8) -> Self {
        self.pri = pri;
        self
    }

    pub fn flags(mut self, flags: u8) -> Self {
        self.flags = flags;
        self
    }

    pub fn src(mut self, src: u8) -> Self {
        self.src = src;
        self
    }

    pub fn dst(mut self, dst: u8) -> Self {
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

impl CspConnection {
    pub fn new() -> Self {
        Self {
            idout: CspId::new(),
            opts: 0,
            state: ConnState::ConnClosed,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn csppacket_test() {
        let test = CspPacket::new();
        assert_eq!(test.frame_begin, [0u8; 4]);
        assert_eq!(test.data, vec![0u8; 0]);
    }

    #[test]
    fn cspid_test() {
        let test = CspId::new()
            .flags(5)
            .pri(2)
            .dport(23)
            .sport(37)
            .src(125)
            .dst(90);
        assert_eq!(test.pri, 2);
        assert_eq!(test.flags, 5);
        assert_eq!(test.src, 125);
        assert_eq!(test.dst, 90);
        assert_eq!(test.dport, 23);
        assert_eq!(test.sport, 37);
    }
}
