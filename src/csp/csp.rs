// SPDX-License-Identifier: MIT

use std::io;

use crate::csp::types::*;

pub struct CSP {
    intf_list: Vec<Box<dyn crate::csp::interface::NextHop>>,
}

impl CSP {

    pub fn new () -> Self {
        CSP {
            intf_list : Vec::new(),
        }
    }

    pub fn add_interface(&mut self, intf: Box<dyn crate::csp::interface::NextHop>) {
        self.intf_list.push(intf);
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

    pub fn csp_read(self, _conn: &mut CspConnection, _timeout: u32) -> Result<CspPacket, io::Error> {
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
    fn send_test() {
        if std::env::args().len() > 1 {
            if std::env::args().nth(1).unwrap() == "nouart" {
                println!("No UART");
                ()
            }
        }

        let _my_csp_id = CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };

        let mut test_int = KissIntfData {
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
            port: None,
        };

        let uart_config = PortConfig {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stopbits: StopBits::One,
        };

        let port = usart_open(uart_config, "/dev/pts/1".to_string());
        test_int.port = port.ok();

        let mut test_conn = CspConnection::new();
        test_conn.state = ConnState::ConnOpen;
        let mut test_pkt = CspPacket::new()
            .data(vec![0, 'a' as u8, 1, 2, 3, 4, 5, 6, 7, 8, 9])
            .length(10);

        let mut csp = CSP::new();
        csp.add_interface(Box::new(test_int));
        let res = csp.csp_send(&mut test_conn, &mut test_pkt);
        //let res = csp_send ( &mut test_conn, &mut test_pkt);
        assert!(res.is_ok());
    }
}
