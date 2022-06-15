// SPDX-License-Identifier: MIT

use crate::csp::types::*;

use std::sync::mpsc::sync_channel;

pub struct Conn {
    arr_conn: Vec<ConnStatus>,
}

#[derive(Debug)]
pub struct ConnStatus {
    pub conn_type: ConnType,
    pub state: ConnState,
    pub idin: Id,
    pub idout: Id,
    pub sport_out: u8,
    pub timestamp: u32,
    pub opts: u32,
    pub channel_rx: std::sync::mpsc::Receiver<CspFIFO>,
    pub channel_tx: std::sync::mpsc::SyncSender<CspFIFO>,
}

#[derive(Debug, PartialEq, Hash, Clone)]
pub enum ConnType {
    ConnClient,
    ConnServer,
}

impl ConnStatus {
    pub fn new() -> Self {
        let (s, r) = sync_channel(16);
        Self {
            conn_type: ConnType::ConnClient,
            state: ConnState::ConnClosed,
            idin: Id::new(),
            idout: Id::new(),
            sport_out: 0,
            timestamp: 0,
            opts: 0,
            channel_rx: r,
            channel_tx: s,
        }
    }
}

impl Conn {
    pub fn new() -> Self {
        info!("CSP conn init");

        let mut ret_val = Conn {
            arr_conn: Vec::new(),
        };

        // TODO: this 16 should be configurable
        for i in 0..16 {
            let mut a = ConnStatus::new();
            a.state = ConnState::ConnClosed;
            a.idin.flags = 0;
            a.sport_out = 16 + i;
            (a.channel_tx, a.channel_rx) = sync_channel(16);
            ret_val.arr_conn.push(a);
        }

        ret_val
    }

    pub fn get(self, idin: Id, idout: Id, typ: ConnType) -> ConnStatus {
        let cspconn = ConnStatus::new();

        cspconn
    }

}

