use std::error::Error;
use std::net::SocketAddr;
async fn listen(addr: &str) -> Result<(), Box<dyn Error>> {
    loop {
        if let Ok(socket_addr) = addr.parse::<SocketAddr>() {
            match socket_addr {
                SocketAddr::V4(_) => {},
                SocketAddr::V6(_) => {},
            }
        }
    }
    Ok(())
}
