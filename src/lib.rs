mod csp;
use csp::{buffer, conn, port, qfifo, types, id};

use serialport::SerialPort;

pub fn csp_init() {
    println!("CSP library init...");

    buffer::csp_buffer_init();

    conn::csp_conn_init();

    port::csp_port_init();

    qfifo::csp_qfifo_init();

    //csp_rdp_queue_init();

    println!("CSP library init... Done");
}

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::ByteOrder;
    use byteorder::{BigEndian, NetworkEndian};

    #[test]
    fn it_works() {
        csp_init();
    }

    #[test]
    fn csp_id1_prepend_test() {

        let my_csp_id = types::CspId {
            pri: 2,
            flags: 1,
            src: 5,
            dst: 12,
            dport: 23,
            sport: 99,
        };

        let mut pkt = types::CspPacket {
            frame_begin: [0;4],
            length : 23,
            id : my_csp_id,
            data : [1;256],
        };

        id::csp_id1_prepend (& mut pkt);
        let mut target : [u8;4] = [0;4];
        NetworkEndian::write_u32(&mut target, 0x8AC5E301);
        //println!("network {} {} {} {} ", target[0], target[1], target[2], target[3]);
        assert_eq!(pkt.frame_begin, target );
    }

}
