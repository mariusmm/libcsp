mod csp;
pub use csp::*;

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

    #[test]
    fn it_works() {
        csp_init();
    }

    

}
