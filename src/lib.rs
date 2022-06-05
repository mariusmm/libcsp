// SPDX-License-Identifier: MIT

pub mod csp;
pub use csp::*;
extern crate pretty_env_logger;
#[macro_use] extern crate log;


pub fn csp_init() {
    //pretty_env_logger::init();
    info!("CSP library init...");

    buffer::csp_buffer_init();

    //conn::csp_conn_init();

    port::csp_port_init();

    qfifo::csp_qfifo_init();

    //csp_rdp_queue_init();


    //let (tx, rx) : (Sender<CspPacket>, Receiver<CspPacket>) = mpsc::channel();
    info!("CSP library init... Done");
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        csp_init();
    }

    

}
