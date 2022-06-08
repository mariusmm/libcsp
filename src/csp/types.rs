// SPDX-License-Identifier: MIT

use std::io;
use crc::{Crc, CRC_32_ISCSI};

use crate::csp::interface::*;

pub const CSPCRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

pub fn csp_send_direct_iface<Intf>(
    _idout: &Id,
    packet: &mut Packet,
    iface: &mut Intf,
    via: u16,
    from_me: bool,
) -> Result<(), io::Error>
where
    Intf: NextHop,
{
    return iface.next_hop(via, packet, from_me);
}

#[derive(Clone,Debug)]
pub struct Packet {
    pub id: Id,
    pub data: Vec<u8>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Id {
    pub pri: u8,
    pub flags: u8,
    pub src: u8,
    pub dst: u8,
    pub dport: u8,
    pub sport: u8,
}

#[derive(PartialEq, Debug)]
pub enum ConnState {
    ConnOpen,
    ConnClosed,
}

pub struct Connection {
    pub opts: u32,
    pub state: ConnState,
    pub idout: Id,
}

#[derive(Debug)]
pub struct CspFIFO {
    pub iface: Iface,
    pub packet: Packet,
}

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    CspNoError,
    CspError,
    CspNoPacket,
}

pub enum Services {
    CMP = 0,
    Ping = 1,
    Ps = 2,
    MemFree = 3,
    Reboot = 4,
    BufFree = 5,
    Uptime = 6,
}

pub enum Priorities {
    PrioCritical,
    PrioHigh,
    PrioNormal,
    PrioLow,
}

impl Packet {
    pub fn new() -> Self {
        Self {
            id: Id::new(),
            data: Vec::new(),
        }
    }

    pub fn id(mut self, id: Id) -> Self {
        self.id = id;
        self
    }

    pub fn data(mut self, data: Vec<u8>) -> Self {
        self.data = data;
        self
    }

    pub fn crc32_append(&mut self) {
        let calc_crc = CSPCRC32.checksum(&mut self.data);

        self.data.push(((calc_crc & 0xFF000000) >> 24) as u8);
        self.data.push(((calc_crc & 0x00FF0000) >> 16) as u8);
        self.data.push(((calc_crc & 0x0000FF00) >> 8) as u8);
        self.data.push((calc_crc & 0x000000FF) as u8);
    }
}

pub fn csp_crc32_calc(data: &Vec<u8>) -> u32 {
    CSPCRC32.checksum(&data)
}

impl Id {
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

impl Connection {
    pub fn new() -> Self {
        Self {
            idout: Id::new(),
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
        let test = Packet::new();
        assert_eq!(test.data, vec![0u8; 0]);
    }

    #[test]
    fn cspid_test() {
        let test = Id::new()
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
