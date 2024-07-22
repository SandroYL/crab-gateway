use std::os::unix::net::UnixStream;

use tokio::{io::BufStream, net::TcpStream};

enum RawStream {
    Tcp(TcpStream),
    Unix(UnixStream),
}

pub struct Stream {
    stream: BufStream<RawStream>,
    buffer_writer: bool,
    proxy_digest: Option<Arc<ProxyDigest>>
}