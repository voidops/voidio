#[cfg(test)]
mod tests {
    use std::net::SocketAddr;
    use voidio::net::QuicClient;

    #[test]
    fn quic_initial () {
        let address: SocketAddr = "127.0.0.1:5003".parse().unwrap();
        let mut client = QuicClient::new("127.0.0.1:5003");
        match client.connect() {
            Ok(_) => println!("Connected to QUIC server at {}", address),
            Err(e) => panic!("Failed to connect to QUIC server: {}", e),
        }
    }
}
