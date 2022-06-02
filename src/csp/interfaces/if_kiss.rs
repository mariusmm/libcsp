// SPDX-License-Identifier: MIT

use std::io;
use std::time::Duration;

use bytes::Bytes;
use serialport::{DataBits, SerialPort, StopBits};

use crate::csp::interface::*;
use crate::csp::types::*;

const FEND: u8 = 0xC0;
const FESC: u8 = 0xDB;
const TFEND: u8 = 0xDC;
const TFESC: u8 = 0xDD;
const TNC_DATA: u8 = 0x00;

#[derive(Clone)]
pub enum CspKissMode {
    KissModeNotStarted, // No start detected
    KissModeStarted,    // Started on a KISS frame
    KissModeEscaped,    // Rx escape character
    KissModeSkipFrame,  // Skip remaining frame, wait for end character
}

pub struct KissIntfData {
    pub intf: CspIface,
    pub max_rx_length: usize,
    pub rx_length: u32,
    pub rx_first: bool,
    pub port: Option<Box<dyn SerialPort>>,
    pub rx_mode: CspKissMode,
}

pub struct PortConfig {
    pub stopbits: StopBits,
    pub baud_rate: u32,
    pub data_bits: DataBits,
}


impl KissIntfData {
    pub fn new(intf: CspIface, config: PortConfig,
        ifname: String,
    ) -> Self {
        let builder = serialport::new(ifname, config.baud_rate)
            .stop_bits(config.stopbits)
            .data_bits(config.data_bits)
            .timeout(Duration::from_millis(10000));
        let p = builder.open().unwrap();

        Self {
            intf: intf,
            max_rx_length: 256,
            rx_mode: CspKissMode::KissModeNotStarted,
            rx_length: 0,
            rx_first: false,
            port:Some(p),
        }
    }

    pub fn csp_kiss_tx(
        self: &KissIntfData,
        _via: u16,
        packet: &mut crate::csp::types::CspPacket,
        _from_me: bool,
    ) -> Result<(), io::Error> {
        println!("Kiss TX {} {}", self.intf.name, packet.data.len());

        packet.csp_crc32_append();

        let flags = packet.id.flags as u8;
        let cspid_low = (packet.id.sport & 0x3F) | (packet.id.dport & 0x3) << 6;
        let cspid_med = ((packet.id.dport & 0x3C) >> 2) | (packet.id.dst & 0x1F) << 4;
        let cspid_high =
            (packet.id.pri << 6) | (packet.id.src & 0x3F) << 1 | (packet.id.dst & 0x10) >> 4;

        packet.data.insert(1, cspid_high);
        packet.data.insert(2, cspid_med);
        packet.data.insert(3, cspid_low);
        packet.data.insert(4, flags);
        packet.data.insert(5, 0x00); //don't now why this extra byte, maybe padding?

        let kiss_buf = kiss_process_tx(&packet.data, packet.data.len());

        //println!("{:x?}", kiss_buf);

        let kiss_len = kiss_buf.len();
        let mem_buff = Bytes::from(kiss_buf);

        match &self.port {
            //TODO: better error management
            None => panic!("Port not initialized for KISS interface"),
            Some(p) => {
                let mut cl = p.try_clone()?;
                cl.write(mem_buff.split_at(kiss_len).0)?;
            }
        };

        Ok(())
    }

    fn usart_rx_func(self: &mut KissIntfData) {
        loop {
            self.csp_kiss_rx().unwrap();
        }
    }

    fn csp_kiss_rx(self: &mut KissIntfData) -> Result<(), io::Error> {

        let mut serial_buf: Vec<u8> = vec![0; self.max_rx_length];

         match &self.port {
            //TODO: better error management
            None => panic!("Port not initialized for KISS interface"),
            Some(p) => {
                let mut cl = p.try_clone()?;

                let r = cl.read(serial_buf.as_mut_slice());
                match r {
                    Ok(t) => {
                        let packet = self.kiss_process_rx(serial_buf, t);

                        if packet.is_ok() {
                            let fifo_pkt = CspFIFO {
                                iface: self.intf.clone(),
                                packet: packet.unwrap().clone(),
                            };

                            if self.intf.rx_channel.is_some() {
                                let res = self.intf.rx_channel.clone().unwrap().send(fifo_pkt);
                            } else {
                                println!("Error no RX FIFO!");
                            }
                        }

                        // Això gasta el rx_channel i el deixa a None !
                        //if let Some(s) = interface.rx_channel.take() {
                        //    s.send(fifo_pkt).unwrap();
                        //}
                        // això no el deu gastar (?)
                        //let mut a= interface.rx_channel.clone();
                        //if let Some(s) = a.take() {
                        //    s.send(fifo_pkt).unwrap();
                        //}

                        return Ok(());
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                    Err(e) => panic!("Error: {:?}", e),
                }
            }
        };
        Ok(())
    }

    fn kiss_process_rx(
        self: &mut KissIntfData,
        data: Vec<u8>,
        len: usize,
    ) -> Result<crate::csp::types::CspPacket, io::Error> {
        let mut n = 0;
        let mut packet = crate::csp::types::CspPacket::new();

        while n < len {
            let inputbyte = data[n];
            n += 1;
            match self.rx_mode {
                CspKissMode::KissModeNotStarted => {
                    if inputbyte != FEND {
                        break;
                    }
                    //csp_id_setup_rx();
                    if packet.data.len() > self.max_rx_length {
                        self.rx_mode = CspKissMode::KissModeSkipFrame;
                    }
                    self.rx_first = true;
                    self.rx_length = 0;
                    self.rx_mode = CspKissMode::KissModeStarted;
                }
                CspKissMode::KissModeStarted => {
                    if inputbyte == FESC {
                        self.rx_mode = CspKissMode::KissModeEscaped;
                        continue;
                    }

                    if inputbyte == FEND {
                        //packet.data.len() == len;
                        // if csp_id_strip <0 error

                        // intf.frame += 1;
                        // validate crc
                        // send to CSP task server using qfifo (?)

                        self.rx_mode = CspKissMode::KissModeNotStarted;
                        continue;
                    }

                    if self.rx_first {
                        self.rx_first = false;
                        continue;
                    }

                    packet.data.push(inputbyte);
                }
                CspKissMode::KissModeEscaped => {
                    if inputbyte == TFESC {
                        packet.data.push(FESC);
                    }

                    if inputbyte == TFEND {
                        packet.data.push(FEND);
                    }
                    self.rx_mode = CspKissMode::KissModeStarted;
                }

                CspKissMode::KissModeSkipFrame => {
                    if inputbyte == FEND {
                        self.rx_mode = CspKissMode::KissModeNotStarted;
                    }
                }
            };
        }
        Ok(packet)
    }
}

impl crate::csp::interface::NextHop for KissIntfData {
    fn next_hop(&self, _via: u16, packet: &mut CspPacket, _from_me: bool) -> Result<(), io::Error> {
        self.csp_kiss_tx(_via, packet, _from_me)
    }
}

pub fn kiss_process_tx(data: &[u8], len: usize) -> Vec<u8> {
    let mut res = Vec::new();

    // start
    res.push(FEND);
    res.push(TNC_DATA);

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
    use crate::csp::csp::*;

    #[test]
    #[ignore]
    pub fn uart() {
        if std::env::args().len() > 1 {
            if std::env::args().nth(1).unwrap() == "nouart" {
                println!("No UART");
                ()
            }
        }

        let port_name = "/dev/pts/0".to_string();
        let builder = serialport::new(port_name, 115200)
            .stop_bits(StopBits::One)
            .data_bits(DataBits::Eight);
        let mut port = builder.open().unwrap();

        let string = "hello world\n".to_string();
        port.write(string.as_bytes()).unwrap();
    }

    #[test]
    #[ignore]
    fn csp_nexthop_test() {
        if std::env::args().len() > 1 {
            if std::env::args().nth(1).unwrap() == "nouart" {
                println!("No UART");
                ()
            }
        }
        let my_csp_id = CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };

        let mut pkt = CspPacket {
            frame_begin: [0; 4],
            id: my_csp_id,
            data: vec![65; 25],
        };

        let mut intf = CspIface::new(12, 5, "KISS".to_string());
        let uart_config = PortConfig {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stopbits: StopBits::One,
        };

        let csp = CSP::new();

        intf.rx_channel = Some(csp.get_rx_channel());

        let mut kiss_intf = KissIntfData::new(intf, uart_config, "/dev/pts/1".to_string());

        let result = csp_send_direct_iface(&my_csp_id, &mut pkt, &mut kiss_intf, 0, false);
        assert!(result.is_ok());
    }

    /// In linux, use the command
    /// ``` > socat -d -d pty,raw,echo=0 pty,raw,echo=0 ```
    /// it will printout what ports to use for the test
    /// One port is used here, use the other in other console with the following command
    /// ``` > echo "helloWorld" > /dev/pty/X
    #[test]
    #[ignore]
    fn csp_uart_rx_test() {
        if std::env::args().len() > 1 {
            if std::env::args().nth(1).unwrap() == "nouart" {
                println!("No UART");
                ()
            }
        }

        let my_csp_id = CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };

        let pkt = CspPacket {
            frame_begin: [0; 4],
            id: my_csp_id,
            data: vec![65; 10],
        };

        let uart_config = PortConfig {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stopbits: StopBits::One,
        };

        let mut intf =  CspIface::new(12, 5, "KISS".to_string());

        let csp = CSP::new();
        intf.rx_channel = Some(csp.get_rx_channel());

        let mut kiss_intf = KissIntfData::new( intf,
            uart_config,
            "/dev/pts/1".to_string(),
        );

        let result = kiss_intf.csp_kiss_rx();
        println!("UART RX: {:#?}", pkt.data);
        println!("Packet len: {}", pkt.data.len());
        assert!(result.is_ok());
    }

    #[test]
    fn csp_kiss_process_rx_test() {


        let mut kiss_intf = KissIntfData {
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
                rx_channel: None,
            },
            max_rx_length: 256,
            rx_mode: CspKissMode::KissModeNotStarted,
            rx_length: 0,
            rx_first: false,
            port: None,
        };

        //let data = vec![0xC0, 0x00, 0x12, 0x34, 0x56, 0x78];
        //let data = vec![0xC0, 0x00, 0x54, 0xDB, 0xDC, 0xDB, 0xDD, 0x53, 0x54, 0xC0];
        let data = vec![0xC0, 0x00, 0xDB, 0xDC, 0xDB, 0xDD];
        let pkt = kiss_intf.kiss_process_rx(data, 6).unwrap();
        println!("len: {} Data: {:#02x?}", pkt.data.len(), pkt.data);
        assert_eq!(pkt.data, vec![0xC0, 0xDB]);
    }
}
