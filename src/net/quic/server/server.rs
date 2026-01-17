use std::net::SocketAddr;
use std::sync::Arc;

use crate::net::{QuicConnectionEvent, UdpServer, UdpServerThreadContext};
use super::{exec_quic_packet, QuicThreadContext};

type OnConnectionEvent = Arc<dyn Fn(QuicConnectionEvent) + Send + Sync + 'static>;

pub enum DispatchMode {
    Direct,
    Async,
}

pub struct QuicServer {
    udp_server: UdpServer,
    datagram_dispatch_mode: DispatchMode,
    onconnection_handler: Option<OnConnectionEvent>,
}

impl QuicServer {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            udp_server: UdpServer::new(address),
            datagram_dispatch_mode: DispatchMode::Direct,
            onconnection_handler: None,
        }
    }

    pub fn udp_server(&self) -> &UdpServer {
        &self.udp_server
    }

    pub fn set_datagram_dispatch_mode(&mut self, mode: DispatchMode) -> &mut Self {
        self.datagram_dispatch_mode = mode;
        self
    }

    pub fn on_connection<H>(&mut self, h: H) -> &mut Self
    where
        H: Fn(QuicConnectionEvent) + Send + Sync + 'static,
    {
        self.onconnection_handler = Some(Arc::new(h));
        self
    }

    pub fn start(&mut self, num_workers: usize) {
        let handler = self
            .onconnection_handler
            .take()
            .expect("on_accept handler must be set before starting the server");
        self.udp_server.thread({
            move |mut udp_ctx: UdpServerThreadContext| {
                let onconnection_handler = handler.clone();
                let mut quic_ctx = QuicThreadContext::new(udp_ctx.id, udp_ctx.socket, onconnection_handler.clone());
                udp_ctx.on_datagram(move |src, data| {
                    if data.len() < 8 {
                        return; // Not enough data for a QUIC packet
                    }
                    let mut i = 0;
                    while i + 7 < data.len() {
                        let pkt = &mut data[i..];
                        let processed_bytes = exec_quic_packet(&mut quic_ctx, pkt, &src);
                        if processed_bytes == 0 {
                            return; // Stop processing if exec_quic_packet returns 0
                        }
                        i += processed_bytes;
                    }
                });
                udp_ctx.run().expect("UDP server thread failed");
            }
        });

        self.udp_server.start(num_workers);
    }

    pub fn stop(&mut self) {
        self.udp_server.stop();
    }

    pub fn is_running(&self) -> bool {
        self.udp_server.is_running()
    }
}
