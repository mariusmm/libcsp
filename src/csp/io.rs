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

