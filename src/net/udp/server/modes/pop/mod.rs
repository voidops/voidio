use crate::dprintln;
use crate::net::*;
impl UdpServerThreadContext {
    pub(crate) fn begin_pop_loop(&mut self) -> std::io::Result<()> {
        use std::sync::atomic::Ordering;
        let mut handler = self.datagram_handler.take().expect("No Datagram handler set for UdpServerThreadContext");
        let mut buf = vec![0u8; 2048];
        let mut buf_len = 0;
        self.make_ready();
        while self.server_running.load(Ordering::Relaxed) {
            match self.socket.popmsg(&mut buf, &mut buf_len, 0) {
                Ok(addr) => {
                    handler(addr, &mut buf[..buf_len]);
                    self.c += 1;
                    if self.c % 100_000 == 0 {
                        self.processed_counter.fetch_add(self.c, Ordering::Relaxed);
                        self.c = 0;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => continue,
                Err(e) => { dprintln!(self, "Error receiving data: {e}"); break }
            }
        }
        Ok(())
    }
}