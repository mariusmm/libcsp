// SPDX-License-Identifier: MIT

use std::io;
use std::time::Duration;

use bytes::Bytes;
use serialport::{DataBits, SerialPort, StopBits};
use byteorder::{ByteOrder};

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
    pub intf: Iface,
    pub port: Option<Box<dyn SerialPort>>,
}

struct KissIntfDataRx {
    pub rx_mode: CspKissMode,
    pub max_rx_length: usize,
    pub rx_length: u32,
    pub rx_first: bool,
}

pub struct PortConfig {
    pub stopbits: StopBits,
    pub baud_rate: u32,
    pub data_bits: DataBits,
}

impl KissIntfData {
    pub fn new(intf: Iface, config: PortConfig,
               ifname: String,
    ) -> Self {
        let builder = serialport::new(&ifname, config.baud_rate)
            .stop_bits(config.stopbits)
            .data_bits(config.data_bits)
            .timeout(Duration::from_millis(10000));
        let p = builder.open().unwrap();
        let q = p.try_clone().unwrap();

        let newval = KissIntfData {
            intf: intf.clone(),
            port:Some(p),
        };

        info!("Creating KISS ({}) interface", ifname);

        std::thread::spawn(move || usart_rx_func( q, &intf.clone()) );

        newval

    }

    pub fn kiss_tx(
        self: &KissIntfData,
        _via: u16,
        packet: &mut crate::csp::types::Packet,
        _from_me: bool,
    ) -> Result<(), io::Error> {
        debug!("Kiss TX {} {}", self.intf.name, packet.data.len());

        packet.crc32_append();

        let flags = packet.id.flags as u8;
        let cspid_low = (packet.id.sport & 0x3F) | (packet.id.dport & 0x3) << 6;
        let cspid_med = ((packet.id.dport & 0x3C) >> 2) | (packet.id.dst & 0x1F) << 4;
        let cspid_high =
            ((packet.id.pri as u8) << 6) | (packet.id.src & 0x3F) << 1 | (packet.id.dst & 0x10) >> 4;

        packet.data.insert(1, cspid_high);
        packet.data.insert(2, cspid_med);
        packet.data.insert(3, cspid_low);
        packet.data.insert(4, flags);
        packet.data.insert(5, 0x00); //don't now why this extra byte, maybe padding?

        let kiss_buf = kiss_process_tx(&packet.data, packet.data.len());
        let kiss_len = kiss_buf.len();
        let mem_buff = Bytes::from(kiss_buf);

        match &self.port {
            //TODO: better error management
            None => {error!("Port not initialized for KISS interface"); panic!("Port not initialized for KISS interface");},
            Some(p) => {
                let mut cl = p.try_clone()?;
                cl.write(mem_buff.split_at(kiss_len).0)?;
            }
        };
        Ok(())
    }
}

impl crate::csp::interface::NextHop for KissIntfData {
    fn next_hop(&self, _via: u16, packet: &mut Packet, _from_me: bool) -> Result<(), io::Error> {
        self.kiss_tx(_via, packet, _from_me)
    }
}

pub fn usart_rx_func(port: Box<dyn SerialPort>, intf: &Iface, ) {
    let mut rx_intf = KissIntfDataRx::new();
    let cl = port.try_clone().unwrap();
    loop {
        let _res = rx_intf.kiss_rx(&cl, intf.clone());
        println!("RX Loop");
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

impl KissIntfDataRx {

    pub fn new() -> Self {
        Self {
            max_rx_length : 256,
            rx_first: true,
            rx_length: 0,
            rx_mode: CspKissMode::KissModeNotStarted,
        }
    }

    fn kiss_rx(self: &mut KissIntfDataRx, port: &Box<dyn SerialPort>, intf: Iface) -> Result<(), io::Error> {

        let mut serial_buf: Vec<u8> = vec![0; self.max_rx_length];
        let mut cl = port.try_clone()?;

        let r = cl.read(serial_buf.as_mut_slice());
        match r {
            Ok(t) => {
                let packet = kiss_process_rx(serial_buf, t, self);
                if packet.is_ok() {
                    let fifo_pkt = CspFIFO {
                        iface: intf.clone(),
                        packet: packet.unwrap().clone(),
                    };

                    if intf.rx_channel.is_some() {
                        let _res = intf.rx_channel.clone().unwrap().send(fifo_pkt);
                    } else {
                        error!("No RX fifo");
                    }
                }

                return Ok(());
            }
            Err(e) => return Err(e),
        };
    }
}

fn kiss_process_rx(
    data: Vec<u8>,
    len: usize,
    intf : &mut KissIntfDataRx,
) -> Result<crate::csp::types::Packet, io::Error> {
    let mut n = 0;
    let mut packet = crate::csp::types::Packet::new();

    while n < len {
        let inputbyte = data[n];
        n += 1;
        match intf.rx_mode {
            CspKissMode::KissModeNotStarted => {
                if inputbyte != FEND {
                    break;
                }
                
                if packet.data.len() > intf.max_rx_length {
                    intf.rx_mode = CspKissMode::KissModeSkipFrame;
                }
                intf.rx_first = true;
                intf.rx_length = 0;
                intf.rx_mode = CspKissMode::KissModeStarted;
            }
            CspKissMode::KissModeStarted => {
                if inputbyte == FESC {
                    intf.rx_mode = CspKissMode::KissModeEscaped;
                    continue;
                }

                if inputbyte == FEND {
                    intf.rx_mode = CspKissMode::KissModeNotStarted;

                    let len = packet.data.len();

                    if len < 9 {
                        warn!("Invalid pkt length");
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "Invalid length"));
                    }

                    debug!("Data: {:x?}", packet.data);
                    let aux_a = packet.data.remove(0);
                    let aux_b = packet.data.remove(0);
                    let aux_c = packet.data.remove(0);
                    let aux_d = packet.data.remove(0);

                    packet.id = get_packet_id(aux_a, aux_b, aux_c, aux_d);

                    debug!("Aux: {:x} {:x} {:x} {:x} ", aux_a, aux_b, aux_c, aux_d);
                    debug!("{:?}", packet.id);

                    // validate crc
                    let len = packet.data.len();

                    let calc_crc_buf = &packet.data[0..len-4].to_vec();
                    let calc_crc = csp_crc32_calc(&calc_crc_buf);

                    let pkt_crc_buf = &packet.data[len-4 ..];
                    let pkt_crc = byteorder::BigEndian::read_u32(pkt_crc_buf);

                    if pkt_crc != calc_crc {
                        warn!("Error CRC");
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, "CRC Error"));
                    } else {
                        debug!("CRC OK!");
                        info!("Accepted packet {:?}", packet.id);
                        return Ok(packet);
                    }
                }

                if intf.rx_first {
                    intf.rx_first = false;
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
                intf.rx_mode = CspKissMode::KissModeStarted;
            }

            CspKissMode::KissModeSkipFrame => {
                if inputbyte == FEND {
                    intf.rx_mode = CspKissMode::KissModeNotStarted;
                }
            }
        };
    }
    Ok(packet)
}

// CSP 1.0
// | Byte0 | Byte 1 | Byte 2 | Byte 3 |
// | 2 PRIO | 5 SOURCE | 5 DESTINATION | 6 DESTINATION PORT | 6 SOURCE PORT | 8 FLAGS |
fn get_packet_id (byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> Id {
    let mut ret_val = Id::new();

    ret_val.sport = byte2 & 0x3F;
    ret_val.dport = byte1 & 0x0F | (byte2 & 0xC0) >> 6;
    ret_val.dst = (byte1 >> 4) | ( byte0 & 0x01 ) << 4;
    ret_val.src = (byte0 >> 1) & 0x1F;
    ret_val.pri = ((byte0 >> 6) & 0x03).into();
    ret_val.flags = byte3;

    ret_val
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
            .data_bits(DataBits::Eight)
            .timeout(Duration::from_millis(1000));
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
        let my_csp_id = Id {
            pri: Priorities::PrioNormal,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };

        let mut pkt = Packet {
            id: my_csp_id,
            data: vec![65; 25],
        };

        let mut intf = Iface::new(12, 5, "KISS".to_string());
        let uart_config = PortConfig {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stopbits: StopBits::One,
        };

        let csp = CSP::new(5);

        intf.rx_channel = Some(csp.get_rx_channel());

        let mut kiss_intf = KissIntfData::new(intf, uart_config, "/dev/pts/5".to_string());

        let result = csp_send_direct_iface(&my_csp_id, &mut pkt, &mut kiss_intf, 0, false);

        assert!(result.is_ok());
    }

    /// In linux, use the command
    /// ``` > socat -d -d pty,raw,echo=0 pty,raw,echo=0 ```
    /// it will printout what ports to use for the test
    #[test]
    #[ignore]
    fn csp_uart_rx_test() {
        
        pretty_env_logger::init();

        if std::env::args().len() > 1 {
            if std::env::args().nth(1).unwrap() == "nouart" {
                println!("No UART");
                ()
            }
        }

        let uart_config = PortConfig {
            baud_rate: 115200,
            data_bits: DataBits::Eight,
            stopbits: StopBits::One,
        };

        let mut intf =  Iface::new(5, 5, "KISS".to_string());

        let mut csp = CSP::new(5);
        intf.rx_channel = Some(csp.get_rx_channel());

        let kiss_intf = KissIntfData::new( intf,
            uart_config,
            "/dev/pts/5".to_string(),
        );

        csp.add_interface(Box::new(kiss_intf));

        std::thread::sleep(std::time::Duration::new(5, 0));
        let pkt = csp.read(Duration::from_millis(10000)).unwrap();
        let data = pkt.data;
        println!("RX packet: {:02X?}", data);
    }

    #[test]
    fn csp_kiss_process_rx_test() {
        //let data = vec![0xC0, 0x00, 0x12, 0x34, 0x56, 0x78];
        //let data = vec![0xC0, 0x00, 0x54, 0xDB, 0xDC, 0xDB, 0xDD, 0x53, 0x54, 0xC0];
        let data = vec![0xC0, 0x00, 0xDB, 0xDC, 0xDB, 0xDD];
        let mut kiss_intf_rx = KissIntfDataRx::new();
        let pkt = kiss_process_rx(data, 6, &mut kiss_intf_rx).unwrap();
        println!("len: {} Data: {:#02x?}", pkt.data.len(), pkt.data);
        assert_eq!(pkt.data, vec![0xC0, 0xDB]);
    }

    #[test]
    fn csp_get_packet_id() {
        let a = get_packet_id(0x82, 0x20, 0x5b, 0x00);
        println!("{:?}", a);
        let cmp = Id {
            pri: Priorities::PrioNormal,
            src: 1,
            dst: 2,
            sport: 27,
            dport: 1,
            flags: 0,
        };
        assert_eq!(a, cmp);
    }
}
