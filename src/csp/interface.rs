// SPDX-License-Identifier: MIT

use std::io;

use crate::csp::types::*;

/**
 * Common data for interfaces. Interfaces must implement its own struct with CspIface inside and the NextHop trait
 */
#[derive(Clone)]
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
    pub rx_channel: Option<std::sync::mpsc::SyncSender<CspFIFO>>,
}

pub trait NextHop {
    fn next_hop(&self, via: u16, packet: &mut CspPacket, from_me: bool) -> Result<(), io::Error>;
}

impl CspIface {
    pub fn new(addr: u16, netmask: u16, name: String) -> CspIface {
        Self {
            addr,
            netmask,
            name,
            mtu: 255,
            split_horizon_off: 0,
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
        }
    }
}
