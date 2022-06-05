// SPDX-License-Identifier: MIT

use crate::csp::types::*;

use std::io;
use std::sync::mpsc::sync_channel;

pub struct CspConn {
    arr_conn: Vec<CspConnStatus>,
}

#[derive(Debug)]
pub struct CspConnStatus {
    pub conn_type: CspConnType,
    pub state: CspConnState,
    pub idin: CspId,
    pub idout: CspId,
    pub sport_out: u8,
    pub timestamp: u32,
    pub opts: u32,
    pub channel_rx: std::sync::mpsc::Receiver<CspFIFO>,
    pub channel_tx: std::sync::mpsc::SyncSender<CspFIFO>,
}

#[derive(Debug)]
pub enum CspConnType {
    CspConnClient,
    CspConnServer,
}

#[derive(Debug)]
pub enum CspConnState {
    CspConnClose,
    CspConnOpen,
}

impl CspConnStatus {
    pub fn new() -> Self {
        let (s, r) = sync_channel(16);
        Self {
            conn_type: CspConnType::CspConnClient,
            state: CspConnState::CspConnClose,
            idin: CspId::new(),
            idout: CspId::new(),
            sport_out: 0,
            timestamp: 0,
            opts: 0,
            channel_rx: r,
            channel_tx: s,
        }
    }
}

impl CspConn {
    pub fn new() -> Self {
        info!("CSP conn init");

        let mut ret_val = CspConn {
            arr_conn: Vec::new(),
        };

        // TODO: this 16 should be configurable
        for i in 0..16 {
            let mut a = CspConnStatus::new();
            a.state = CspConnState::CspConnClose;
            a.idin.flags = 0;
            a.sport_out = 16 + i;
            (a.channel_tx, a.channel_rx) = sync_channel(16);
            ret_val.arr_conn.push(a);
        }

        ret_val
    }

    pub fn get(self, idin: CspId, idout: CspId, typ: CspConnType) -> CspConnStatus {
        let cspconn = CspConnStatus::new();

        cspconn
    }

}

pub fn csp_connect(
    prio: CspPriorities,
    dest: u16,
    dport: u8,
    timeout: u32,
    opts: u8,
) -> Result<CspConnection, io::Error> {
    let a = CspConnection {
        opts: 0,
        state: ConnState::ConnClosed,
        idout: CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        },
    };

    Ok(a)
}
