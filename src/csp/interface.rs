// SPDX-License-Identifier: MIT

use std::io;

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
    fn next_hop(&self, via: u16, packet: &mut CspPacket, from_me: bool) -> Result<(), io::Error>;
}
