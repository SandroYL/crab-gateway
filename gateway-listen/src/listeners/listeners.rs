use std::io;

use tokio::net::TcpListener;

use super::stream::Stream;
pub enum Listener {
    Tcp(TcpListener),
    Unix(),
}

impl Listener {
    pub async fn accept(&self) -> io::Result<Stream> {
        todo!()
    }
}