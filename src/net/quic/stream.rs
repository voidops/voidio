use std::net::SocketAddr;

use super::connection::QuicConnection;

pub struct QuicStream<'a> {
    pub(crate) id: u32,
    pub(crate) src: &'a QuicConnection,
    pub(crate) ondata_handler: Option<Box<dyn FnMut(&[u8]) + Send>>,
    pub(crate) onclose_handler: Option<Box<dyn FnMut() + Send>>,
}

impl<'a> QuicStream<'a> {
    pub fn new(id: u32, src: &'a QuicConnection) -> Self {
        Self {
            id,
            src,
            ondata_handler: None,
            onclose_handler: None,
        }
    }

    pub fn id(&self) -> u32 {
        self.id
    }

    pub fn src_connection(&self) -> &QuicConnection {
        self.src
    }

    pub fn write(&self, data: &[u8]) -> Result<(), String> {
        Err("QuicStream::write is not implemented yet".to_string())
    }

    pub fn on_data<F>(&mut self, h: F) where F: FnMut(&[u8]) + Send + 'static {
        self.ondata_handler = Some(Box::new(h));
    }

    pub fn read(&self, buf: &mut [u8]) -> Result<usize, String> {
        Err("QuicStream::read is not implemented yet".to_string())
    }

    pub fn on_close<F>(&mut self, h: F) where F: FnMut() + Send + 'static {
        self.onclose_handler = Some(Box::new(h));
    }

    pub fn src(&self) -> SocketAddr {
        self.src.address
    }
}

impl<'a> std::fmt::Display for QuicStream<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuicStream(id: {}, src: {})", self.id, self.src.address)
    }
}