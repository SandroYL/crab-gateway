pub enum ListenerAddress {
    Tcp(String),
    Udp(String),
}

impl AsRef<str> for ListenerAddress {
    fn as_ref(&self) -> &str {
        match &self {
            ListenerAddress::Tcp(addr) => &addr,
            ListenerAddress::Udp(addr) => &addr,
        }
    }
}