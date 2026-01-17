use std::net::{SocketAddr, ToSocketAddrs};
use rand::RngCore;
use ring::hkdf::{Salt, HKDF_SHA256};
use rustls::pki_types::ServerName;
use crate::net::{AfInet, AfInet6, IpProtoUdp, SockDgram, Socket, INITIAL_SALT};
use crate::net::connection::QuicConnection;
use crate::net::quic::packets::{build_initial_packet};
use super::{QuicStream};

pub struct QuicClient {
    server_name: ServerName<'static>,
    addr: SocketAddr,
    socket: Socket,
    connection: Option<QuicConnection>,
    onopen_handler: Option<Box<dyn FnMut(& mut QuicConnection) + Send>>,
}

impl QuicClient {
    pub fn new(host: &'_ str) -> Self {
        let server_name = ServerName::try_from(host.split(":").next().unwrap().to_owned()).expect("invalid server name");
        let addr = host
            .to_socket_addrs()
            .unwrap()
            .next()
            .unwrap();
        let socket = if addr.is_ipv4() {
            Socket::new(AfInet, SockDgram, IpProtoUdp).unwrap()
        } else {
            Socket::new(AfInet6, SockDgram, IpProtoUdp).unwrap()
        };
        Self {
            server_name,
            addr,
            socket,
            connection: None,
            onopen_handler: None,
        }
    }
    pub fn connect(&mut self) -> Result<(), String> {
        let mut scid = vec![0u8; 8];
        let mut dcid = vec![0u8; 20];
        rand::rng().fill_bytes(&mut scid);
        rand::rng().fill_bytes(&mut dcid);
        let salt = Salt::new(HKDF_SHA256, &INITIAL_SALT);
        //let client_hello = build_quic_client_hello(self.server_name.clone(), &dcid, &[b"h3"]);
        //let packet = build_initial_packet(&salt, &dcid, &scid, client_hello.as_ref());
        //self.socket.send_to(packet.as_ref(), &self.addr, 0).map_err(|e| e.to_string())?;
        Ok(())
    }
    
    pub fn on_open<F>(&mut self, h: F)
    where
        F: FnMut(& mut QuicConnection) + Send + 'static,
    {
        self.onopen_handler = Some(Box::new(h));
    }

    pub fn open_bistream(&'_ mut self) -> Result<QuicStream<'_>, String> {
        if let Some(connection) = &mut self.connection {
            connection.open_bistream()
        } else {
            Err("Connection not established".to_string())
        }
    }
}