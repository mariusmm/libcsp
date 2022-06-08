// SPDX-License-Identifier: MIT

use std::io;
use std::sync::mpsc::sync_channel;
use std::time::Duration;

use crate::csp::types::*;

///
/// ## CSP
///
/// Outside -> intf's -> CSP Routing -> modules (csp_read) : get_rx_channel
///
/// Modules (csp_send) -> CSP Routing -> Intf's -> Outside)
///
pub struct CSP {
    intf_list: std::sync::Arc<std::sync::Mutex<Vec<Box<dyn crate::csp::interface::NextHop>>>>,
    inb_channel_out: Option<std::sync::mpsc::SyncSender<CspFIFO>>,
    outb_channel_in: std::sync::Arc<std::sync::Mutex<std::sync::mpsc::Receiver<CspFIFO>>>,
    outb_channel_out: std::sync::mpsc::SyncSender<CspFIFO>,
}

impl CSP {
    pub fn new() -> Self {
        // TODO: Any better style to keep tuple at init time?
        let (a, b) = sync_channel(16);

        let ret = CSP {
            intf_list: std::sync::Arc::new(std::sync::Mutex::new(Vec::new())),
            inb_channel_out: None,
            outb_channel_in: std::sync::Arc::new(std::sync::Mutex::new(b)),
            outb_channel_out: a,
        };
        ret.start_routing();

        ret
    }

    pub fn add_interface(&mut self, intf: Box<dyn crate::csp::interface::NextHop>) {
        self.intf_list.lock().unwrap().push(intf);
    }

    pub fn get_rx_channel(&self) -> std::sync::mpsc::SyncSender<CspFIFO> {
        self.outb_channel_out.clone()
    }

    pub fn csp_send(
        &self,
        conn: &mut CspConnection,
        packet: &mut CspPacket,
    ) -> Result<(), io::Error> {
        if conn.state != ConnState::ConnOpen {
            warn!("Connection closed");
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Connection Closed",
            ))?
        }

        self.csp_send_direct(conn, packet)
    }

    pub fn csp_send_direct(
        &self,
        _conn: &mut CspConnection,
        packet: &mut CspPacket,
    ) -> Result<(), io::Error> {
        let from_me = true;
        let via = 2u16;

        /*self.intf_list.lock().and_then(|item| {
            item[0].next_hop(via, packet, from_me);
            Ok(())
        } ).unwrap(); */

        let a = self.intf_list.lock().unwrap();
        a[0].next_hop(via, packet, from_me)

        //iface.next_hop(via, packet, from_me)
    }

    /* TODO: Implement! */
    pub fn csp_read(&self, _timeout: Duration) -> Result<CspPacket, CspError> {
       /* let pkt = self.outb_channel_in.recv_timeout(timeout);
        match pkt {
            Ok(p) => Ok(p.packet),
            Err(_) => return Err(crate::csp::types::CspError::CspNoPacket),
        }
        */
        Ok(CspPacket::new())
    }

    fn start_routing(&self) 
    {
        info!("Start routing");
        let ch = self.outb_channel_in.clone(); 

        std::thread::spawn( move|| {
            let a = ch.lock().unwrap().recv();
            // match a {
            //     Ok(p) => debug!("Routing rcv: {:?} \n\t {:?}", p.packet, p.iface),
            //     _ => {debug!("Error! {:?}", a); println!("Error aqui")},
            // }
            let data = a.unwrap();
            debug!("ROUTE RX: {:?}\n\t{:?}", data.iface, data.packet);

            //let ch = self.outb_channel_out.clone();
            //ch.send(data);

            //let ifaces = self.intf_list.lock().and_then(|mut item| {
            //    Ok(item)
            //} ).unwrap();
        });
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
