//! Implementation of an UDP packet "stream".
//! Never closes, i.e., will try to read indefinitely.
use std::io::Read;
use std::net::ToSocketAddrs;
use std::net::UdpSocket;

/// Simple implementation of an UDP packet "stream".
///
/// This is somewhat of an workaround since UDP does not really represent
/// a stream and just independent network packages. This "stream" never closes so
/// any reader will try to read indefinitely.
pub struct UdpStream {
    socket: UdpSocket,
}

impl UdpStream {
    /// Binds member `socket` to supplied address.
    pub fn open<T: ToSocketAddrs>(addr: T) -> std::io::Result<Self> {
        Ok(UdpStream {
            socket: UdpSocket::bind(addr)?,
        })
    }
}

impl Read for UdpStream {
    /// Reads a packet into supplied buffer.
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        self.socket.recv(buf)
    }
}
