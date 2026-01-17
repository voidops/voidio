use std::{collections::HashMap, sync::Arc};

use ring::hkdf::{Salt, HKDF_SHA256};
use crate::net::{QuicLongHeader, Socket, INITIAL_SALT};
use crate::net::connection::{ConnectionId, QuicConnection};

pub struct QuicConnectionEvent<'a> {
    pub connection: &'a mut QuicConnection,
}

pub type OnConnectionEvent<'a> = Arc<dyn Fn(QuicConnectionEvent) + Send + Sync + 'static>;

pub struct QuicThreadContext<'a> {
    pub(crate) id: usize,
    pub(crate) udp_socket: Socket,
    pub(crate) connections: HashMap<ConnectionId, QuicConnection>,
    pub(crate) initial_salt: Salt,
    pub(crate) client_init_buf: [u8; 32],
    pub(crate) hp_key_buf: [u8; 16],
    pub(crate) aead_key_buf: [u8; 16],
    pub(crate) aead_iv_buf:  [u8; 12],
    pub(crate) curr_long_hdr: QuicLongHeader<'a>,
    pub(crate) onconnection_handler: OnConnectionEvent<'a>,
    pub(crate) debug_mode: bool,
}

impl<'a> QuicThreadContext<'a> {
    pub fn new(id: usize, udp_socket: Socket, onconnection_handler: OnConnectionEvent<'a>) -> Self {
        Self {
            id,
            udp_socket,
            connections: HashMap::new(),
            initial_salt: Salt::new(HKDF_SHA256, &INITIAL_SALT),
            client_init_buf: [0; 32],
            hp_key_buf: [0; 16],
            aead_key_buf: [0; 16],
            aead_iv_buf: [0; 12],
            curr_long_hdr: QuicLongHeader {
                flags: 0,
                version: 0,
                dcid: &[],
                scid: &[],
                header_size: 0,
            },
            onconnection_handler,
            debug_mode: true,
        }
    }

    pub(crate) fn send_udp_packet(&self, buf: &[u8], addr: &std::net::SocketAddr) -> std::io::Result<usize> {
        self.udp_socket.send_to(buf, addr, 0)
    }

    pub(crate) fn notify_on_connection(&self, conn: &mut QuicConnection) {
        println!("New QUIC connection: {}[{}] => {}[Server]", conn.id, conn.address, conn.dcid);
        (self.onconnection_handler)(QuicConnectionEvent {
            connection: conn,
        });
    }
}