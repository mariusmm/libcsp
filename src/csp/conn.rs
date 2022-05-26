// SPDX-License-Identifier: MIT

use crate::csp::types::*;
use std::io;

pub fn csp_conn_init() {
    println!("CSP conn init");
}

pub fn csp_connect(_prio: CspPriorities, _dest: u16, _dport: u8, _timeout: u32, _opts: u8) -> Result<CspConnection, io::Error> {

    let a =  CspConnection {
        opts : 0,
        state : ConnState::ConnClosed,
        idout : CspId {
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