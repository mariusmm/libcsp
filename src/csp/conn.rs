// SPDX-License-Identifier: MIT

use crate::csp::types::*;

use std::io;
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

#[derive(Debug)]
pub enum ConnType {
    CspConnClient,
    CspConnServer,
}

impl ConnStatus {
    pub fn new() -> Self {
        let (s, r) = sync_channel(16);
        Self {
            conn_type: ConnType::CspConnClient,
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

pub fn csp_connect(
    prio: Priorities,
    dest: u16,
    dport: u8,
    timeout: u32,
    opts: u8,
) -> Result<Connection, io::Error> {
    let a = Connection {
        opts: 0,
        state: ConnState::ConnClosed,
        idout: Id {
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
