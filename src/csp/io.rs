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

    if conn.state != ConnState::ConnOpen{
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Connection Closed"))?
    }
    
    csp_send_direct (conn, packet)
}

pub fn csp_send_direct (_conn:&mut CspConnection, _packet: &mut CspPacket) -> Result<(), io::Error> {
    let _from_me = true;

    Ok(())
}

pub fn csp_read (_conn: &mut CspConnection, _timeout: u32 ) -> Result<CspPacket, io::Error> {
    let ret = CspPacket::new();
    Ok(ret)
}

pub fn csp_connect(_prio: CspPriorities, _dest: u16, _dport: u8, _timeout: u32, _opts: u8) -> Result<CspConnection, io::Error> {

    let a =  CspConnection {
        opts : 0,
        state : ConnState::ConnClosed,
    };

    Ok(a)
}