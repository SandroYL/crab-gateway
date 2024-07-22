use std::io;

use tokio::net::{TcpListener, UnixListener};
pub enum Listener {
    Tcp(TcpListener),
    Unix(UnixListener),
}

impl Listener {
    pub async fn accept(&self) -> io::Result<Stream>
}