use std::net::TcpStream;

pub mod scan_port {
    use std::net::{TcpStream, UdpSocket};

    pub fn any(port: u16) -> bool {
        panic!("This function (TCP port scanner) does not work properly!");
    }

    // if true, the port is occupied.
    pub fn tcp(port: u16) -> bool {
        panic!("This function (TCP port scanner) does not work properly!");
        match TcpStream::connect(("0.0.0.0", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    // if true, the port is occupied.
    pub fn udp(port: u16) -> bool {
        panic!("This function (UDP port scanner) does not work properly!");
        match UdpSocket::bind(format!("0.0.0.0:{}", port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }
}
