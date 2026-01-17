use std::{net::SocketAddr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc::Sender, Arc}};

use crate::net::*;

#[macro_export]
macro_rules! dprintln {
    ($self:expr, $fmt:literal $(, $args:expr)* $(,)?) => {
        if $self.debug_mode {
            println!($fmt $(, $args)*);
        }
    };
}

pub struct UdpServerThreadContext {
    pub(crate) id: usize,
    pub(crate) name: String,
    pub(crate) server_address: SocketAddr,
    pub(crate) server_running: Arc<AtomicBool>,
    pub(crate) debug_mode: bool,
    pub(crate) socket: Socket,
    pub(crate) c: usize,
    pub(crate) processed_counter: Arc<AtomicUsize>,
    pub(crate) datagram_handler: Option<Box<dyn FnMut(SocketAddr, &mut [u8]) + Send + Sync + 'static>>,
    pub(crate) kernel_mode: bool,
    pub(crate) ready_tx: Sender<()>,
}

impl UdpServerThreadContext {
    pub fn new(socket: Socket, server_address: SocketAddr, ready_tx: Sender<()>) -> Self {
        Self {
            id: 0,
            name: format!("UdpServerThreadContext-X"),
            server_address,
            socket,
            server_running: Arc::new(AtomicBool::new(false)),
            debug_mode: false,
            c: 0,
            processed_counter: Arc::new(AtomicUsize::new(0)),
            datagram_handler: None,
            kernel_mode: false,
            ready_tx
        }
    }

    #[inline(always)]
    pub fn pop(&mut self, buf: &mut [u8], len: &mut usize) -> std::io::Result<SocketAddr> {
        if self.c % 100_000 == 0 {
            self.processed_counter.fetch_add(self.c, Ordering::Relaxed);
            self.c = 0;
        }
        self.socket.popmsg(buf, len, 0)
    }

    #[inline(always)]
    pub fn send(&self, buf: &[u8], addr: &SocketAddr) -> std::io::Result<usize> {
        self.socket.send_to(buf, addr, 0)
    }

    pub fn on_datagram<F>(&mut self, h: F)
    where
        F: FnMut(SocketAddr, &mut [u8]) + Send + Sync + 'static,
    {
        self.datagram_handler = Some(Box::new(h));
    }

    pub fn make_ready(&self) {
        self.ready_tx.send(()).expect("Failed to send ready signal");
        while !self.server_running.load(Ordering::Relaxed) {
            std::thread::yield_now();
        }
    }

    pub fn run(&mut self) -> std::io::Result<()> {
        if self.kernel_mode {
            self.begin_raw_queue_poll_loop()
        } else {
            #[cfg(unix)] {
                self.begin_popmany_loop()
            }
            #[cfg(not(unix))] {
                self.begin_pop_loop()
            }
        }
    }
}