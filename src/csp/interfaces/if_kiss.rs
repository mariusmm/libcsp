// SPDX-License-Identifier: MIT

use std::io;
use std::time::Duration;

use bytes::Bytes;
use serialport::{DataBits, StopBits, SerialPort};

use crate::csp::interface::*;
use crate::csp::types::*;


const FEND:u8 = 0xC0;
const FESC:u8 = 0xDB;
const TFEND:u8 = 0xDC;
const TFESC:u8 = 0xDD;
const TNC_DATA:u8 = 0x00;

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
    pub fn new(intf:  CspIface) -> Self {
        Self {
            intf: intf,
            max_rx_length: 256,
            rx_mode: CspKissMode::KissModeNotStarted,
            rx_length: 0,
            rx_first: false,
            port : None,
        }
    }

    pub fn csp_kiss_tx(
        self: &KissIntfData,
        _via: u16,
        packet: &mut crate::csp::types::CspPacket,
        _from_me: bool,
    ) -> Result<(), io::Error> {
        println!("Kiss TX {} {}", self.intf.name, packet.length);
    
        //let length = csp_crc32_append(& mut packet.data, packet.length);
        let length = packet.length;
        let kiss_buf = kiss_process_tx(&packet.data, length);
        let mem_buff = Bytes::from(kiss_buf);
     
        match &self.port {
            //TODO: better error management
            None => panic!("Port not initialized for KISS interface"), 
            Some (p) => {
                let mut cl = p.try_clone()?;
                cl.write(mem_buff.split_at(packet.length).0)?;
            }
        };
        Ok(())
    }
}

impl crate::csp::interface::NextHop for KissIntfData {
    fn next_hop(& self, _via: u16, packet: &mut CspPacket, _from_me: bool) -> Result<(), io::Error> {
        self.csp_kiss_tx(_via, packet, _from_me)
    }
}

pub fn usart_open(
    kissintf: &mut KissIntfData,
    config: PortConfig,
    ifname: String,
) -> Result<(), io::Error> {
    let builder = serialport::new(ifname, config.baud_rate)
        .stop_bits(config.stopbits)
        .data_bits(config.data_bits)
        .timeout(Duration::from_millis(10000));
    let p = builder.open()?;

    kissintf.port = Some(p);
    
    Ok(())
}

pub fn csp_kiss_rx (interface: &mut KissIntfData,
    packet: &mut crate::csp::types::CspPacket) -> Result<(), io::Error> {

        let mut serial_buf: Vec<u8> = vec![0; interface.max_rx_length];
        match &interface.port {

            //TODO: better error management
            None => panic!("Port not initialized for KISS interface"), 
            Some (p) => {
                let mut cl = p.try_clone()?;

                    let r = cl.read(serial_buf.as_mut_slice());
                    match r {
                        Ok(t) => {
                            kiss_process_rx (serial_buf, t, packet, interface).unwrap();
                            return Ok(());
                        },
                        Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
                        Err(e) => panic!("Error: {:?}", e)
                    }
            }
        };
        Ok(())
}



fn kiss_process_tx(data: &[u8], len: usize) -> Vec<u8> {

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

fn kiss_process_rx(data: Vec<u8>
    , len: usize
    , packet: &mut crate::csp::types::CspPacket
    , intf: &mut KissIntfData,
    ) -> Result<(), io::Error> {
    let mut n = 0;
    while n < len {
        let inputbyte = data[n];
        n += 1;
        match intf.rx_mode {
            CspKissMode::KissModeNotStarted => {
                if inputbyte != FEND {
                    break;
                }
                //csp_id_setup_rx();
                if packet.length > intf.max_rx_length {
                    intf.rx_mode = CspKissMode::KissModeSkipFrame;
                }
                intf.rx_first = true;
                intf.rx_length = 0;
                intf.rx_mode = CspKissMode::KissModeStarted;
            },
            CspKissMode::KissModeStarted => {
                if inputbyte == FESC {
                    intf.rx_mode = CspKissMode::KissModeEscaped;
                    continue;
                }

                if inputbyte == FEND {
                    packet.length = len;
                    // if csp_id_strip <0 error
                    
                    // intf.frame += 1;
                    // validate crc
                    // send to CSP task server using qfifo (?)

                    intf.rx_mode = CspKissMode::KissModeNotStarted;
                    continue;
                }

                if intf.rx_first {
                    intf.rx_first = false;
                    continue;
                }

                packet.data.push(inputbyte);
            },
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
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn uart() {
        if std::env::args().len() > 1 {
            if  std::env::args().nth(1).unwrap() == "nouart" {
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
    fn csp_nexthop_test() {
        if std::env::args().len() > 1 {
            if  std::env::args().nth(1).unwrap() == "nouart" {
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
             length: 25,
             id: my_csp_id,
             data: vec![65; 256],
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
             baud_rate : 115200,
             data_bits : DataBits::Eight,
             stopbits : StopBits::One,
         };
        
        usart_open(&mut test_int, uart_config, "/dev/pts/0".to_string()).unwrap();
        let result = csp_send_direct_iface(&my_csp_id, &mut pkt, &mut test_int, 0, false);
        assert! (result.is_ok());
     }

     
    /// In linux, use the command
    /// ``` > socat -d -d pty,raw,echo=0 pty,raw,echo=0 ```
    /// it will printout what ports to use for the test
    /// One port is used here, use the other in other console with the following command
    /// ``` > echo "helloWorld" > /dev/pty/X
    #[test]
    fn csp_uart_rx_test() {
        if std::env::args().len() > 1 {
            if  std::env::args().nth(1).unwrap() == "nouart" {
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
             length: 25,
             id: my_csp_id,
             data: vec![65; 1],
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
             baud_rate : 115200,
             data_bits : DataBits::Eight,
             stopbits : StopBits::One,
         };
        
        usart_open(&mut test_int, uart_config, "/dev/pts/5".to_string()).unwrap();
        let result = csp_kiss_rx(&mut test_int, &mut pkt);
        println!("UART RX: {:#?}", pkt.data);
        println!("Packet len: {}", pkt.length);
        assert! (result.is_ok());
     }

     #[test]
     fn csp_kiss_process_rx_test () {

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
            length: 25,
            id: my_csp_id,
            data: Vec::new(),
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

        //let data = vec![0xC0, 0x00, 0x12, 0x34, 0x56, 0x78];
        //let data = vec![0xC0, 0x00, 0x54, 0xDB, 0xDC, 0xDB, 0xDD, 0x53, 0x54, 0xC0];
        let data = vec![0xC0, 0x00, 0xDB, 0xDC, 0xDB, 0xDD];
        kiss_process_rx (data, 6, &mut pkt, &mut test_int).unwrap();
        println!("len: {} Data: {:#02x?}", pkt.length, pkt.data);
        assert_eq!(pkt.data, vec![0xC0,0xDB]);
     }
}
