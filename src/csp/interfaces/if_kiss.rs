
use crate::csp::types::*;
use crate::csp::interface::*;

use serialport::SerialPort;

pub enum CspKissMode {
    KissModeNotStarted,  // No start detected
	KissModeStarted,      // Started on a KISS frame
	KissModeEscaped,      // Rx escape character 
	KissModeSkipFrame,   // Skip remaining frame, wait for end character
}

pub struct KissIntfData{
    pub intf: CspIface,
    pub max_rx_length: u32,
    pub rx_mode : CspKissMode,
    pub rx_length: u32,
    pub rx_first: bool,
}

impl crate::csp::interface::NextHop for KissIntfData {
    fn next_hop (&self, via: u16, packet: & mut CspPacket, from_me: u32) -> Result<(), CspError> {
        csp_kiss_tx (&self, via, packet, from_me)
    }
}

pub fn csp_kiss_tx (iface: &KissIntfData, via: u16, packet : &mut crate::csp::types::CspPacket, from_me: u32) -> Result<(), CspError>
{
    println!("Kiss TX {} {}", iface.intf.name, packet.length);
    Ok(())
}


