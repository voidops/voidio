#[cfg(test)]
mod tests {
    use std::net::Ipv4Addr;
    use voidio::net::UdpServer;

    #[test]
    fn test_udp_server_flood () {
        std::thread::sleep(std::time::Duration::from_millis(1));
        let address = "127.0.0.1:42070".parse().unwrap();
        let mut server = UdpServer::new(address);
        server.thread(move |mut ctx| {
            ctx.on_datagram(move |src, data| {
                /*
                if src.port() != 42070 && !data.starts_with(b"Hello, world!") {
                    println!("Received packet from unexpected address: {} with data: {:?}", src, String::from_utf8_lossy(data));
                    return;
                }*/
            });
            ctx.run().expect("Failed to run server thread");
        });
        server.debug(true);
        server.start(1).expect("Failed to start server");
        assert!(server.is_running());
        // Send 1 million packets to the server
        server.floodtest(42070)
            .with_threads(8)
            .with_payload_size(1200)
            .with_duration(std::time::Duration::from_secs(5))
            .with_logs(true)
            .start();
        server.stop();
        server.wait(None);
        assert!(!server.is_running());
    }
}
