use std::{fmt::Display, mem::MaybeUninit};
use super::{SockAddrIn, SockAddrIn6, TimeVal};

#[derive(Clone, Copy)]
pub struct SocketAddrSrcV4(SockAddrIn);

impl SocketAddrSrcV4 {
    pub fn new(addr: SockAddrIn) -> Self {
        SocketAddrSrcV4(addr)
    }
    pub fn raw(&self) -> &SockAddrIn {
        &self.0
    }
    pub fn from_raw_parts(addr: u64) -> Self {
        return SocketAddrSrcV4(SockAddrIn {
            sin_family: 2, // AF_INET
            sin_port: (addr as u16).to_be(),
            #[cfg(unix)]
            sin_addr: super::sys::InAddr { s_addr: (addr >> 32) as u32 },
            #[cfg(windows)]
            sin_addr: super::sys::InAddr { s_addr: addr.to_be_bytes()[0..4].try_into().unwrap() },
            sin_zero: [0; 8],
        });
    }
    pub fn from_socket_addr(addr: &std::net::SocketAddrV4) -> Self {
        SocketAddrSrcV4(SockAddrIn {
            sin_family: 2, // AF_INET
            sin_port: addr.port().to_be(),
            #[cfg(unix)]
            sin_addr: super::sys::InAddr { s_addr: u32::from_be_bytes(addr.ip().octets()) },
            #[cfg(windows)]
            sin_addr: super::sys::InAddr { s_addr: addr.ip().octets().try_into().unwrap() },
            sin_zero: [0; 8],
        })
    }
    #[inline(always)]
    pub fn to_ipv4_addr(&self) -> std::net::Ipv4Addr {
        self.0.to_ipv4_addr()
    }
    #[inline(always)]
    pub fn to_socket_addr_v4(&self) -> std::net::SocketAddrV4 {
        self.0.to_socket_addr_v4()
    }
    pub fn to_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::V4(self.to_socket_addr_v4())
    }
}

impl Display for SocketAddrSrcV4 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_socket_addr_v4())
    }
}

#[derive(Clone, Copy)]
pub struct SocketAddrSrcV6(SockAddrIn6);

impl SocketAddrSrcV6 {
    pub fn new(addr: SockAddrIn6) -> Self {
        SocketAddrSrcV6(addr)
    }
    pub fn raw(&self) -> &SockAddrIn6 {
        &self.0
    }
    pub fn from_socket_addr(addr: &std::net::SocketAddrV6) -> Self {
        SocketAddrSrcV6(SockAddrIn6 {
            sin6_family: 10, // AF_INET6
            sin6_port: addr.port().to_be(),
            sin6_flowinfo: 0,
            #[cfg(unix)]
            sin6_addr: super::sys::In6Addr { s6_addr: addr.ip().octets() },
            #[cfg(windows)]
            sin6_addr: super::sys::In6Addr { s6_addr: addr.ip().octets().try_into().unwrap() },
            sin6_scope_id: 0,
        })
    }
    #[inline(always)]
    pub fn to_ipv6_addr(&self) -> std::net::Ipv6Addr {
        self.0.to_ipv6_addr()
    }
    #[inline(always)]
    pub fn to_socket_addr_v6(&self) -> std::net::SocketAddrV6 {
        self.0.to_socket_addr_v6()
    }
    pub fn to_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::V6(self.to_socket_addr_v6())
    }
}

// pack
#[repr(C)]
pub enum SocketAddrSrc {
    V4(SocketAddrSrcV4),
    V6(SocketAddrSrcV6),
}

impl SocketAddrSrc {
    pub fn new(addr: SockAddrIn) -> Self {
        SocketAddrSrc::V4(SocketAddrSrcV4::new(addr))
    }
    
    pub fn new_v6(addr: SockAddrIn6) -> Self {
        SocketAddrSrc::V6(SocketAddrSrcV6::new(addr))
    }

    pub fn from_socket_addr(addr: &std::net::SocketAddr) -> Self {
        match addr {
            std::net::SocketAddr::V4(v4) => SocketAddrSrc::V4(SocketAddrSrcV4::from_socket_addr(v4)),
            std::net::SocketAddr::V6(v6) => SocketAddrSrc::V6(SocketAddrSrcV6::from_socket_addr(v6)),
        }
    }
}

impl ToSocketAddr for SocketAddrSrc {
    fn to_socket_addr(&self) -> std::io::Result<std::net::SocketAddr> {
        match self {
            SocketAddrSrc::V4(addr) => Ok(std::net::SocketAddr::V4(addr.to_socket_addr_v4())),
            SocketAddrSrc::V6(addr) => Ok(std::net::SocketAddr::V6(addr.to_socket_addr_v6())),
        }
    }
}

pub trait ToSocketAddr {
    fn to_socket_addr(&self) -> std::io::Result<std::net::SocketAddr>;
}

pub trait SocketAddressBuffer {
    type RawSockAddr;

    #[inline(always)]
    fn as_raw_ptr(&self) -> *const Self::RawSockAddr {
        self as *const _ as *const Self::RawSockAddr
    }

    #[inline(always)]
    fn as_raw_mut_ptr(&mut self) -> *mut Self::RawSockAddr {
        self as *mut _ as *mut Self::RawSockAddr
    }

    #[inline(always)]
    fn as_raw(&self) -> &Self::RawSockAddr {
        unsafe { &*self.as_raw_ptr() }
    }

    #[inline(always)]
    fn as_ptr(&self) -> *const u8 {
        self as *const _ as *const u8
    }

    #[inline(always)]
    fn as_mut_ptr(&mut self) -> *mut u8 {
        self as *mut _ as *mut u8
    }

    #[inline(always)]
    fn len(&self) -> usize {
        std::mem::size_of::<Self::RawSockAddr>()
    }
    
}
pub trait ToIpv4Addr {
    fn to_ipv4_addr(&self) -> std::net::Ipv4Addr;
    fn to_socket_addr_v4(&self) -> std::net::SocketAddrV4;
    fn to_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::V4(self.to_socket_addr_v4())
    }
}
pub trait ToIpv6Addr {
    fn to_ipv6_addr(&self) -> std::net::Ipv6Addr;
    fn to_socket_addr_v6(&self) -> std::net::SocketAddrV6;
    fn to_socket_addr(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::V6(self.to_socket_addr_v6())
    }
}
pub trait SockaddrConvert {
    fn to_socket_addr(&self) -> std::io::Result<std::net::SocketAddr>;
    fn addr_len(&self) -> usize;
}

pub trait SocketAddrV4IntoSockAddrV4Buffer {
    fn into_sockaddrv4(&self) -> SockAddrIn;
}

pub trait SocketAddrV6IntoSockAddrV6Buffer {
    fn into_sockaddrv6(&self) -> SockAddrIn6;
}

pub trait TimeValFromDuration {
    fn from_duration(duration: std::time::Duration) -> TimeVal;
}

impl SocketAddressBuffer for SockAddrIn {
    type RawSockAddr = SockAddrIn;
}

impl SocketAddressBuffer for SockAddrIn6 {
    type RawSockAddr = SockAddrIn6;
}

impl SocketAddressBuffer for SocketAddrSrcV4 {
    type RawSockAddr = SockAddrIn;
}

impl SocketAddressBuffer for SocketAddrSrcV6 {
    type RawSockAddr = SockAddrIn6;
}

impl SocketAddressBuffer for MaybeUninit<SockAddrIn> {
    type RawSockAddr = SockAddrIn;
}

impl SocketAddressBuffer for MaybeUninit<SockAddrIn6> {
    type RawSockAddr = SockAddrIn6;
}
