// SPDX-License-Identifier: MIT

use crc::{Crc, CRC_32_ISCSI};

pub const CSPCRC32: Crc<u32> = Crc::<u32>::new(&CRC_32_ISCSI);

pub fn csp_crc32_append(data: &mut Vec<u8>, len:  usize) -> usize {
    let calc_crc = CSPCRC32.checksum(data);

    data.push( ((calc_crc & 0xFF000000) >> 24) as u8);
    data.push(((calc_crc & 0x00FF0000) >> 16) as u8);
    data.push(((calc_crc & 0x0000FF00) >> 8) as u8);
    data.push((calc_crc & 0x000000FF) as u8);
    
    len+4
}
