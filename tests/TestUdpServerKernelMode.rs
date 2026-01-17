#[cfg(test)]
mod tests {
    use std::sync::{atomic::{AtomicUsize, Ordering}, Arc};
    use voidio::net::UdpServer;

    #[test]
    fn udp_server () {
        std::thread::sleep(std::time::Duration::from_millis(1));
        let total_requests = Arc::new(AtomicUsize::new(0));
        let address = "127.0.0.1:42070".parse().unwrap();
        let mut server = UdpServer::new(address);
        server.thread(move |mut ctx| {
            let mut worker_counter = 0;
            let global_counter = total_requests.clone();
            ctx.on_datagram(move |src, data| {
                if !data.starts_with(b"Hello, world!") {
                    println!("Received packet from unexpected address: {} with data: {:?}", src, String::from_utf8_lossy(data));
                    return;
                }
                worker_counter += 1;
                if worker_counter % 100_000 == 0 {
                    global_counter.fetch_add(worker_counter, Ordering::Relaxed);
                    worker_counter = 0;
                }
            });
        });
        server.debug(true);
        server.start(1).unwrap();
        assert!(server.is_running());
        // Send 1 million packets to the server
        server.floodtest(42070)
            .with_threads(8)
            .with_payload_size(64)
            .with_duration(std::time::Duration::from_secs(5))
            .with_logs(true)
            .start();
        server.stop();
        server.wait(None);
        assert!(!server.is_running());
    }
}
