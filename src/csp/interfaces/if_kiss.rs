use std::io;
use bytes::Bytes;
use serialport::{DataBits, StopBits};

use crate::csp::interface::*;
use crate::csp::types::*;

#[allow(dead_code)]
pub enum CspKissMode {
    KissModeNotStarted, // No start detected
    KissModeStarted,    // Started on a KISS frame
    KissModeEscaped,    // Rx escape character
    KissModeSkipFrame,  // Skip remaining frame, wait for end character
}

pub struct KissIntfData {
    pub intf: CspIface,
    pub max_rx_length: u32,
    pub rx_mode: CspKissMode,
    pub rx_length: u32,
    pub rx_first: bool,
    pub port_name: String,
}

impl crate::csp::interface::NextHop for KissIntfData {
    fn next_hop(&self, _via: u16, packet: &mut CspPacket, _from_me: u32) -> Result<(), io::Error> {
        csp_kiss_tx(&self, _via, packet, _from_me)
    }
}

pub fn csp_kiss_tx(
    iface: &KissIntfData,
    _via: u16,
    packet: &mut crate::csp::types::CspPacket,
    _from_me: u32,
) -> Result<(), io::Error> {
    println!("Kiss TX {} {}", iface.intf.name, packet.length);

    let port_name = "/dev/pts/0".to_string();
    let builder = serialport::new(port_name, 115200)
        .stop_bits(StopBits::One)
        .data_bits(DataBits::Eight);
    let mut port = builder.open().unwrap();

    let kiss_buf = kiss_process(&packet.data, packet.length);

    //let mem_buff = Bytes::from_static(packet.data);

    let mem_buff = Bytes::from(kiss_buf);
    port.write(mem_buff.split_at(packet.length).0)?;

    Ok(())
}

fn kiss_process(data: &[u8], len: usize) -> Vec<u8> {

    const FEND:u8 = 0xC0;
    const FESC:u8 = 0xDB;
    const TFEND:u8 = 0xDC;
    const TFESC:u8 = 0xDD;
    const TNC_DATA:u8 = 0x00;

    let mut res = Vec::new();

    // start
    res.push (FEND);
    res.push (TNC_DATA);

    for n in 1..len {
        if data[n] == FEND {
            res.push(FESC);
            res.push(TFEND);
            continue;
        }
        if data[n] == FESC {
            res.push(FESC);
            res.push(TFESC);
            continue;
        }
        res.push(data[n]);
    }
    // stop
    res.push(FEND);
    
    return res;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csp::io::*;

    #[test]
    pub fn uart() {
        let port_name = "/dev/pts/0".to_string();
        let builder = serialport::new(port_name, 115200)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight);
        let mut port = builder.open().unwrap();

        let string = "hello world".to_string();
        port.write(string.as_bytes()).unwrap();
    }

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
             length: 25,
             id: my_csp_id,
             data: [65; 256],
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
             port_name: "/dev/pts/0".to_string(),
         };
 
         let _result = csp_send_direct_iface(&my_csp_id, &mut pkt, &test_int, 0, 1);
         //assert_eq! (result, Ok(()))
     }
}
