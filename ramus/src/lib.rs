use std::io;
use std::net::{IpAddr, SocketAddr, TcpListener, ToSocketAddrs};
use std::str::FromStr;

pub mod http;

pub struct Server {
    #[allow(dead_code)]
    listener: TcpListener,
}

impl Server {
    /// Creates an instance of the Server bound to a given

    pub fn bind<A: ToSocketAddrs>(addrs: A) -> io::Result<Self> {
        TcpListener::bind(addrs).map(|listener| Self { listener })
    }

    pub fn bind_to_default_port(ip_addr: &str) -> io::Result<Self> {
        match IpAddr::from_str(ip_addr) {
            Ok(addr) => Self::bind(SocketAddr::from((addr, 80))),
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err)),
        }
    }
}
