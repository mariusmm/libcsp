use crate::csp::interface::*;
use crate::csp::types::*;

pub fn csp_send_direct_iface<Intf>(
    _idout: &CspId,
    packet: &mut CspPacket,
    iface: &Intf,
    _via: u16,
    from_me: u32,
) -> Result<(), CspError>
where
    Intf: NextHop,
{
    iface.next_hop(_via, packet, from_me)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csp::interfaces::if_kiss::*;

    #[test]
    fn csp_nexthop_test() {
       let my_csp_id = CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };
        
        println!("Deb: {} {} {}", my_csp_id.dport, my_csp_id.flags, my_csp_id.dst);

        let mut pkt = CspPacket {
            frame_begin: [0; 4],
            length: 55,
            id: my_csp_id,
            data: [1; 256],
        };

        let test_int = KissIntfData {
            intf: CspIface {
                addr: 12,
                netmask: 5,
                name: "KISS".to_string(),
                mtu: 7,
                split_horizon_off: 1,
                tx: 0,
                rx: 0,
                tx_error: 0,
                rx_error: 0,
                drop: 0,
                autherr: 0,
                frame: 0,
                txbytes: 0,
                rxbytes: 0,
                irq: 0,
            },
            max_rx_length: 256,
            rx_mode: CspKissMode::KissModeNotStarted,
            rx_length: 0,
            rx_first: false,
        };

        let _result = csp_send_direct_iface(&my_csp_id, &mut pkt, &test_int, 0, 1);
        //assert_eq! (result, Ok(()))
    }
}
