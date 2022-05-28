// SPDX-License-Identifier: MIT

use super::*;
use byteorder::ByteOrder;
use byteorder::NetworkEndian;

//pub const CSP_ID1_PRIO_MASK: u8 = 0x3;
pub const CSP_ID1_PRIO_OFFSET: u8 = 30;
//pub const CSP_ID1_SRC_MASK: u8 = 0x1F;
pub const CSP_ID1_SRC_OFFSET: u8 = 25;
//pub const CSP_ID1_DST_MASK: u8 = 0x1F;
pub const CSP_ID1_DST_OFFSET: u8 = 20;
//pub const CSP_ID1_DPORT_MASK: u8 = 0x3F;
pub const CSP_ID1_DPORT_OFFSET: u8 = 14;
//pub const CSP_ID1_SPORT_MASK: u8 = 0x3F;
pub const CSP_ID1_SPORT_OFFSET: u8 = 8;
//pub const CSP_ID1_FLAGS_MASK: u8 = 0xFF;
pub const CSP_ID1_FLAGS_OFFSET: u8 = 0;

pub fn csp_id1_prepend(packet: &mut types::CspPacket) {
    let id1: u32 = ((packet.id.pri as u32) << CSP_ID1_PRIO_OFFSET)
        | ((packet.id.dst as u32) << CSP_ID1_DST_OFFSET)
        | ((packet.id.src as u32) << CSP_ID1_SRC_OFFSET)
        | ((packet.id.dport as u32) << CSP_ID1_DPORT_OFFSET)
        | ((packet.id.sport as u32) << CSP_ID1_SPORT_OFFSET)
        | ((packet.id.flags as u32) << CSP_ID1_FLAGS_OFFSET);

    NetworkEndian::write_u32(&mut packet.frame_begin, id1);
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn csp_id1_prepend_test() {

        let my_csp_id = types::CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };

        let mut pkt = types::CspPacket {
            frame_begin: [0;4],
            id : my_csp_id,
            data : vec![1;256],
        };

        id::csp_id1_prepend (& mut pkt);
        let mut target : [u8;4] = [0;4];
        NetworkEndian::write_u32(&mut target, 0x8AC5E301);
        //println!("network {} {} {} {} ", target[0], target[1], target[2], target[3]);
        assert_eq!(pkt.frame_begin, target );
    }
}