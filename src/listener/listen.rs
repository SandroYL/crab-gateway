use std::error::Error;
use std::net::{SocketAddr, ToSocketAddrs};

use tokio::net::TcpSocket;
async fn listen(addr: &str) -> Result<(), Box<dyn Error>> {
    loop {
        addr.to_socket_addrs()
        .map_err(||)
         {
            match socket_addr {
                SocketAddr::V4(_) => TcpSocket::new_v4(),
                SocketAddr::V6(_) => TcpSocket::new_v6(),
            }
        }
    }
    Ok(())
}
