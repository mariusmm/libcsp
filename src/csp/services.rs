// SPDX-License-Identifier: MIT

use crate::csp::conn::*;
use crate::csp::csp::*;
use crate::csp::types::*;

impl CSP {
    pub fn csp_ping(
        self,
        node: u16,
        timeout: u32,
        conn_options: u8,
    ) -> Result<(), CspError> {
        let mut conn = csp_connect(
            CspPriorities::CspPrioNormal,
            node,
            CspServices::CspPing as u8,
            timeout,
            conn_options,
        )
        .unwrap();

        let mut packet = CspPacket::new();

        let mut idx = 0;
        for a in packet.data.iter_mut() {
            *a = idx;
            idx += 1;
        }

        self.csp_send(&mut conn, &mut packet).unwrap();

       // let _spacket = self.csp_read(&mut conn, timeout).unwrap();

        // check echo

        Ok(())
    }
}
