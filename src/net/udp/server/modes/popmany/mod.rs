use crate::net::*;
impl UdpServerThreadContext {
    #[cfg(unix)]
    pub(crate) fn begin_popmany_loop(&mut self) -> std::io::Result<()> {
        use std::sync::atomic::Ordering;
        let mut handler = self.datagram_handler.take().expect("No Datagram handler set for XUdpServerWorker");
        let drain_capacity = 8;
        let mut bucket = Ipv4Bucket::new(drain_capacity, 2048);
        let bucket_ref = &mut bucket;
        self.make_ready();
        while self.server_running.load(Ordering::Relaxed) {
            match self.socket.vecrecv(bucket_ref, 0) {
                Ok(count) => {
                    for i in 0..count {
                        let (addrv4, buf) = unsafe { bucket_ref.unsafe_peek(i) };
                        let addr = addrv4.to_socket_addr();
                        handler(addr, buf);
                    }
                    self.c += count;
                    if self.c % 10_000 == 0 {
                        self.processed_counter.fetch_add(self.c, Ordering::Relaxed);
                        self.c = 0;
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock || e.kind() == std::io::ErrorKind::TimedOut => {
                    continue;
                }
                Err(e) => {
                    dprintln!(self, "Error receiving data: {e}");
                    break;
                }
            }
        }
        Ok(())
    }
}