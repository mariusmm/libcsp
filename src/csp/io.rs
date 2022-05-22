use crate::csp::interface::*;
use crate::csp::types::*;
use std::io;

pub fn csp_send_direct_iface<Intf>(
    _idout: &CspId,
    packet: &mut CspPacket,
    iface: &mut Intf,
    _via: u16,
    from_me: u32,
) -> Result<(), io::Error>
where
    Intf: NextHop,
{
    return iface.next_hop(_via, packet, from_me);
}

pub fn csp_send (conn:&mut CspConnection, packet: &mut CspPacket) -> Result<(), io::Error> {

    Ok(())
}

pub fn csp_read (conn: &mut CspConnection, timeout: u32 ) -> Result<CspPacket, io::Error> {
    let ret = CspPacket::new();
    Ok(ret)
}

pub fn csp_connect(prio: CspPriorities, dest: u16, dport: u8, timeout: u32, opts: u8) -> Result<CspConnection, io::Error> {

    let a =  CspConnection {
        opts : 0,
    };

    Ok(a)
}