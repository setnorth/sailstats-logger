use std::io::Read;
use std::net::UdpSocket;
use std::net::ToSocketAddrs;

pub struct UdpStream{
    socket: UdpSocket
}
impl UdpStream{
    pub fn open<T: ToSocketAddrs>(addr: T) -> std::io::Result<Self>{
        Ok(UdpStream{socket: UdpSocket::bind(addr)?})
    }
}
impl Read for UdpStream{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize>{
        self.socket.recv(buf)
    }
}