use std::io;
use std::time::Duration;

use crate::csp::interfaces::if_kiss::*;
use crate::csp::types::*;

/**
 * Common data for interfaces. Interfaces must implement its own struct with CspIface inside and the NextHop trait
 */
pub struct CspIface {
    pub addr: u16,
    pub netmask: u16,
    pub name: String,
    pub mtu: u16,
    pub split_horizon_off: u8,
    pub tx: u32,
    pub rx: u32,
    pub tx_error: u32,
    pub rx_error: u32,
    pub drop: u32,
    pub autherr: u32,
    pub frame: u32,
    pub txbytes: u32,
    pub rxbytes: u32,
    pub irq: u32,
}

pub trait NextHop {
    fn next_hop(&mut self, via: u16, packet: &mut CspPacket, from_me: u32)
        -> Result<(), io::Error>;
}

pub fn usart_open(
    kissintf: &mut KissIntfData,
    config: PortConfig,
    ifname: String,
) -> Result<(), io::Error> {
    let builder = serialport::new(ifname, config.baud_rate)
        .stop_bits(config.stopbits)
        .data_bits(config.data_bits)
        .timeout(Duration::from_millis(100));
    let p = builder.open()?;

    kissintf.port = Some(p);

    Ok(())
}
