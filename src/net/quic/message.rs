use std::net::SocketAddr;
use std::sync::Weak;
use super::connection::QuicConnection;

pub struct QuicMessage {
    pub(crate) src: Weak<QuicConnection>,
    pub(crate) data: Vec<u8>,
}

impl QuicMessage {
    pub fn new(src: Weak<QuicConnection>, data: Vec<u8>) -> Self {
        Self { src, data }
    }

    pub fn src(&self) -> Option<SocketAddr> {
        self.src.upgrade().map(|c| c.address)
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

impl std::fmt::Display for QuicMessage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.src() {
            Some(addr) => write!(f, "QuicMessage(src: {}, data: {:?})", addr, self.data),
            None => write!(f, "QuicMessage(src: <dropped>, data: {:?})", self.data),
        }
    }
}
