use std::net::SocketAddr;

use super::connection::QuicConnection;

pub struct QuicDatagram<'a> {
    pub(crate) src: &'a QuicConnection,
    pub(crate) data: Vec<u8>,
}

impl<'a> QuicDatagram<'a> {
    pub fn new(src: &'a QuicConnection, data: Vec<u8>) -> Self {
        Self {
            src,
            data,
        }
    }

    pub fn src(&self) -> SocketAddr {
        self.src.address
    }
    pub fn data(&self) -> &[u8] {
        &self.data
    }
    pub fn into_data(self) -> Vec<u8> {
        self.data
    }
    pub fn str(&self) -> String {
        String::from_utf8_lossy(&self.data).to_string()
    }
}

impl<'a> std::fmt::Display for QuicDatagram<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuicDatagram(src: {}, data: {:?})", self.src.address, self.data)
    }
}