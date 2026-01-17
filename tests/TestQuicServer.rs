#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use std::thread;
    use std::time::Duration;

    use voidio::net::{DispatchMode, QuicConnectionEvent, QuicServer};

    fn handle_connections(event: QuicConnectionEvent) {
        let client = event.connection;
        println!("New connection from: {}", client.address());
        /*
        client.on_stream(|stream| {
            println!("New stream opened: {}", stream);
            stream.on_data(|data| {
                println!("Received stream data: {:?}", data);
            });
        });*/
    }

    #[test]
    fn test_quic_server_with_quinn_client() {
        let address: SocketAddr = "127.0.0.1:4433".parse().unwrap();

        // Start server in a background thread
        thread::spawn(move || {
            println!("Starting QUIC server on {}", address);
            let mut server = QuicServer::new(address);
            server.set_datagram_dispatch_mode(DispatchMode::Direct);
            server.on_connection(handle_connections);
            server.start(4);
        });

        thread::sleep(Duration::from_millis(500)); // let server start
        println!("Test finished");
    }
}
