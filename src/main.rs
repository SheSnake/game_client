extern crate tokio;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, TcpStream, Shutdown};
use std::io::Write;
use tokio::net::TcpListener;
use tokio::prelude::*;


fn main() {
    println!("Hello, world!");
    let mut stream = TcpStream::connect("0.0.0.0:8890");
    if let Ok(mut stream) = stream {
        let buf: u8 = 1;
        let len: i32 = 32;
        stream.write(&buf.to_le_bytes());
        stream.write(&len.to_le_bytes());
    }
    while true {
        break;
    }
}
