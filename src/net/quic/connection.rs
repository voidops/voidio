use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use super::{QuicDatagram, QuicMessage, QuicStream};

pub enum QuicConnectionState {
    Open,
    Closing,
    Closed,
}

pub enum QuicConnectionType {
    Client,
    Server,
}

#[derive(Clone, Copy, Eq)]
pub struct ConnectionId {
    pub(crate) id: [u8; 20],
    pub(crate) len: usize,
}

impl ConnectionId {
    #[inline(always)]
    pub fn from_slice(slice: &[u8]) -> Self {
        let mut id = [0u8; 20];
        let len = slice.len().min(20);
        id[..len].copy_from_slice(&slice[..len]);
        Self { id, len }
    }

    #[inline(always)]
    pub fn as_bytes(&self) -> &[u8] {
        &self.id[..self.len]
    }
}

impl std::fmt::Display for ConnectionId {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_str: String = self.id[..self.len].iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "{}", hex_str)
    }
}

impl std::fmt::Debug for ConnectionId {
    #[inline(always)]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hex_str: String = self.id[..self.len].iter().map(|b| format!("{:02x}", b)).collect();
        write!(f, "ConnectionId({})", hex_str)
    }
}


impl std::hash::Hash for ConnectionId {
    #[inline(always)]
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.id[..self.len].hash(state);
    }
}

impl PartialEq for ConnectionId {
    #[inline(always)]
    fn eq(&self, other: &Self) -> bool {
        self.len == other.len && self.id[..self.len] == other.id[..other.len]
    }
}

pub struct QuicConnection {
    pub(crate) id: ConnectionId,
    pub(crate) dcid: ConnectionId,
    //pub(crate) state: QuicConnectionState,
    pub(crate) last_packet_number: u32, // Packet Number
    pub(crate) address: SocketAddr,
    pub(crate) onstream_handler: Option<Box<dyn FnMut(&mut QuicStream) + Send + Sync + 'static>>,
    onmessage_handler: Option<
        Box<dyn FnMut(QuicMessage) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>,
    >,
    pub(crate) ondatagram_handler: Option<Box<dyn FnMut(&QuicDatagram) + Send + Sync + 'static>>,
    pub(crate) onclose_handler: Option<Box<dyn FnMut(&QuicConnection) + Send + Sync + 'static>>,
    pub(crate) last_bistream_id: u32,
    pub(crate) last_unistream_id: u32,
}

impl QuicConnection {
    pub fn new(scid: ConnectionId, dcid: ConnectionId, last_packet_number: u32, address: &SocketAddr, type_: QuicConnectionType) -> Self {
        match type_ {
            QuicConnectionType::Client => {
                Self {
                    id: scid,
                    dcid,
                    last_packet_number,
                    address: *address,
                    onstream_handler: None,
                    onmessage_handler: None,
                    ondatagram_handler: None,
                    onclose_handler: None,
                    last_bistream_id: 0, // Client-Initiated, Bidirectional: starts at 0
                    last_unistream_id: 2, // Client-Initiated, Unidirectional: starts at 2
                }
            }
            QuicConnectionType::Server => {
                Self {
                    id: scid,
                    dcid,
                    last_packet_number,
                    address: *address,
                    onstream_handler: None,
                    onmessage_handler: None,
                    ondatagram_handler: None,
                    onclose_handler: None,
                    last_bistream_id: 1, // Server-Initiated, Bidirectional: starts at 1
                    last_unistream_id: 3, // Server-Initiated, Unidirectional: starts at 3
                }
            }
        }
    }

    pub fn initiate_message_channel(&mut self) {
    }
    pub fn open_bistream(&mut self) -> Result<QuicStream, String>{
        self.last_bistream_id += 4;
        Ok(QuicStream::new(
            self.last_bistream_id,
            self,
        ))
    }

    pub fn on_stream<F>(&mut self, h: F)
    where
        F: FnMut(&mut QuicStream) + Send + Sync + 'static,
    {
        self.onstream_handler = Some(Box::new(h));
    }

    pub fn on_message<F, Fut>(&mut self, mut h: F)
    where
        F: FnMut(QuicMessage) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.onmessage_handler = Some(Box::new(move |msg| {
            Box::pin(h(msg))
        }));
    }
    pub async fn trigger_message(&mut self, msg: QuicMessage) {
        if let Some(handler) = self.onmessage_handler.as_mut() {
            handler(msg).await;
        }
    }
    pub fn on_datagram<F>(&mut self, h: F)
    where
        F: FnMut(&QuicDatagram) + Send + Sync + 'static,
    {
        self.ondatagram_handler = Some(Box::new(h));
    }

    pub fn on_close<F>(&mut self, h: F)
    where
    F: FnMut(&QuicConnection) + Send + Sync + 'static {
        self.onclose_handler = Some(Box::new(h));
    }

    #[inline(always)]
    pub fn address(&self) -> &SocketAddr {
        &self.address
    }
}

impl std::fmt::Display for QuicConnection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "QuicConnection(id: {}, address: {})", self.id, self.address)
    }
}

pub struct QuicConnectionEvent<'a> {
    pub connection: &'a mut QuicConnection,
}