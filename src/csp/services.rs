use crate::csp::types::*;
use crate::csp::io::*;

pub fn csp_ping( node: u16, timeout: u32, size: usize, conn_options: u8) -> Result<(), CspError>{

    let mut conn = csp_connect(CspPriorities::CspPrioNormal, node, CspServices::CspPing as u8, timeout, conn_options).unwrap();

    let mut packet = CspPacket::new();

    packet.length = size;
    let mut idx = 0;
    for a in packet.data.iter_mut() {
        *a = idx;
        idx += 1;
    }

    csp_send(&mut conn, &mut packet);

    packet = csp_read(&mut conn, timeout).unwrap();

    // check echo

    Ok(())
}