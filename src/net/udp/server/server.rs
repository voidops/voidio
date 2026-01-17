use std::{
    net::SocketAddr, sync::{atomic::{AtomicBool, AtomicUsize, Ordering}, mpsc, Arc}, time::Duration
};
use crate::dprintln;
use crate::net::*;

pub struct UdpServer {
    pub(crate) running: Arc<AtomicBool>,
    address: SocketAddr,
    threads: Vec<std::thread::JoinHandle<()>>,
    thread_handler: Option<Arc<dyn Fn(UdpServerThreadContext) + Send + Sync + 'static>>,
    debug_mode: bool,
    pub(crate) processed_packets: Vec<Arc<AtomicUsize>>,
    pub total_processed_packets: Arc<AtomicUsize>,
}

impl UdpServer {
    pub fn new(address: SocketAddr) -> Self {
        Self {
            running: Arc::new(AtomicBool::new(false)),
            address,
            threads: Vec::new(),
            thread_handler: None,
            debug_mode: false,
            processed_packets: Vec::new(),
            total_processed_packets: Arc::new(AtomicUsize::new(0)),
        }
    }
    
    pub fn start(&mut self, num_workers: usize) -> std::io::Result<()> {
        if self.thread_handler.is_none() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "No worker handler set",
            ));
        }
        dprintln!(self, "[UdpServer] Starting at {}", self.address);
        let (ready_tx, ready_rx) = mpsc::channel();
        for id in 0..num_workers {
            if id > 0 {
                ready_rx.recv().unwrap();
            }
            let counter = Arc::new(AtomicUsize::new(0));
            let thread_name = format!("UdpServer->Thread-{id}");
            let thread = std::thread::Builder::new()
                .name(thread_name.clone())
                .spawn({
                    let name = thread_name.clone();
                    let debug = self.debug_mode.clone();
                    let running = self.running.clone();
                    let address = self.address.clone();
                    let counter = counter.clone();
                    let ready_signal = ready_tx.clone();
                    let context_setup_handler = self.thread_handler.as_ref().map(Arc::clone);
                    move || {
                        let socket = (if address.is_ipv4() {
                            Socket::new(AfInet, SockDgram, IpProtoUdp)
                        } else {
                            Socket::new(AfInet6, SockDgram, IpProtoUdp)
                        }).unwrap();
                        socket.set_socket_option(SoRecvBufSize, 32768).expect("Failed to set SoRecvBufSize");
                        socket.set_socket_option(SoRecvTimeout, Duration::from_millis(500)).expect("Failed to set SoRecvTimeout");
                        socket.set_socket_option(SoReuseAddr, true).expect("Failed to set SoReuseAddr");
                        socket.bind(&address).expect("Failed to bind socket");

                        let mut ctx = UdpServerThreadContext::new(socket, address, ready_signal);
                        ctx.id = id;
                        ctx.name = name;
                        ctx.processed_counter = counter.clone();
                        ctx.server_running = running;
                        ctx.debug_mode = debug;
                        
                        if debug { println!("[{}] Started", thread_name); }

                        if let Some(context_setup_handler) = context_setup_handler {
                            context_setup_handler(ctx);
                        } else {
                            panic!("No thread handler set for UdpServer");
                        }
                    }
                })?;
            self.threads.push(thread);
            self.processed_packets.push(counter.clone());
        }
        ready_rx.recv().unwrap();
        self.running.store(true, Ordering::Relaxed);
        // Statistics thread
        let processed_packets = self.processed_packets.clone();
        let total = self.total_processed_packets.clone();
        std::thread::Builder::new()
            .name("UdpServer->Stats".to_string())
            .spawn(move || {
                loop {
                    std::thread::sleep(Duration::from_millis(250));
                    total.store(processed_packets.iter().map(|c| c.load(Ordering::Relaxed)).sum(), Ordering::Relaxed);
                }
            })?;
        drop(ready_tx);
        Ok(())
    }
    
    pub fn set_address(&mut self, address: SocketAddr) {
        self.address = address;
    }

    pub fn get_address(&self) -> SocketAddr {
        self.address
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    pub fn thread<WorkerSetupHandler>(&mut self, handler: WorkerSetupHandler) -> &mut Self
    where
        WorkerSetupHandler: Fn(UdpServerThreadContext) + Send + Sync + 'static {
        self.thread_handler = Some(Arc::new(handler));
        self
    }

    pub fn stop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        for thread in self.threads.drain(..) {
            let _ = thread.join();
        }
    }
    
    pub fn debug(&mut self, debug: bool) -> &mut Self {
        self.debug_mode = debug;
        self
    }

    pub fn wait(&mut self, interval: Option<Duration>) {
        let iv = interval.unwrap_or(Duration::from_millis(10));
        while self.running.load(Ordering::Relaxed) {
            std::thread::sleep(iv);
        }
    }

    pub fn worker_count(&self) -> usize {
        self.threads.len()
    }

    pub fn floodtest(&'_ self, local_port: u16) -> UdpFloodTest<'_> {
        UdpFloodTest::new(&self, local_port)
    }
}
unsafe impl Sync for UdpServer {}
