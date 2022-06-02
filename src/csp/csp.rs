// SPDX-License-Identifier: MIT

use std::io;
use std::sync::mpsc::sync_channel;


use crate::csp::types::*;

pub struct CSP {
    intf_list: Vec<Box<dyn crate::csp::interface::NextHop>>,
    channel_rx: std::sync::mpsc::Receiver<CspFIFO>,
    channel_tx: std::sync::mpsc::SyncSender<CspFIFO>,
}

impl CSP {
    pub fn new() -> Self {
        // TODO: Any better style to keep tuple at init time?
        // TODO: This 16 should be configurable
        let (a, b) = sync_channel(16);
        CSP {
            intf_list: Vec::new(),
            channel_tx: a,
            channel_rx: b,
        }
    }

    pub fn add_interface(&mut self, intf: Box<dyn crate::csp::interface::NextHop>) {
        self.intf_list.push(intf);
    }

    pub fn get_rx_channel(&self) -> std::sync::mpsc::SyncSender<CspFIFO> {
        self.channel_tx.clone()
    }
    pub fn csp_send(
        self,
        conn: &mut CspConnection,
        packet: &mut CspPacket,
    ) -> Result<(), io::Error> {
        if conn.state != ConnState::ConnOpen {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Connection Closed",
            ))?
        }

        self.csp_send_direct(conn, packet)
    }

    pub fn csp_send_direct(
        self,
        _conn: &mut CspConnection,
        packet: &mut CspPacket,
    ) -> Result<(), io::Error> {
        let from_me = true;
        let via = 2u16;

        let iface = &self.intf_list[0];

        iface.next_hop(via, packet, from_me)
    }

    pub fn csp_read(
        self,
        _conn: &mut CspConnection,
        _timeout: u32,
    ) -> Result<CspPacket, io::Error> {
        let ret = CspPacket::new();
        Ok(ret)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::csp::interface::*;
    use crate::csp::interfaces::if_kiss::*;
    use serialport::{DataBits, StopBits};

    #[test]
    #[ignore]
    fn send_test() {
        if std::env::args().len() > 1 {
            if std::env::args().nth(1).unwrap() == "nouart" {
                println!("No UART");
                ()
            }
        }

        let test_csp_id = CspId {
            pri: 2,
            flags: 0,
            src: 1,
            dst: 8,
            dport: 1,
            sport: 36,
        };

        let mut intf = CspIface::new(12, 5, "KISS".to_string());

        let uart_config = PortConfig {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stopbits: StopBits::One,
        };

        let mut test_conn = CspConnection::new();
        test_conn.state = ConnState::ConnOpen;

        let mut csp = CSP::new();
        intf.rx_channel = Some(csp.get_rx_channel());

        let kiss_intf = KissIntfData::new(intf, uart_config, "/dev/pts/4".to_string());
        csp.add_interface(Box::new(kiss_intf));

        let mut test_pkt = CspPacket::new()
        .data(vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9])
        .id(test_csp_id);

        let res = csp.csp_send(&mut test_conn, &mut test_pkt);
        assert!(res.is_ok());
    }
}
