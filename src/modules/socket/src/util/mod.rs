use std::net::TcpStream;

pub mod scan_port {
    use std::net::{TcpStream, UdpSocket};

    // if true, the port is occupied.
    pub fn tcp(port: u16) -> bool {
        match TcpStream::connect(("0.0.0.0", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    // if true, the port is occupied.
    pub fn udp(port: u16) -> bool {
        match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
