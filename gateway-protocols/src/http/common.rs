use core::fmt::Debug;
use std::any::Any;
use tokio::io::{AsyncRead, AsyncWrite};

pub(super) const MAX_HEADERS: usize = 256;

pub(super) const INIT_HEADER_BUF_SIZE: usize = 4096;
pub(super) const MAX_HEADER_SIZE: usize = 1048575;

pub(super) const BODY_BUF_LIMIT: usize = 1024 * 64;

pub const CRLF: &[u8; 2] = b"\r\n";
pub const HEADER_KV_DELIMITER: &[u8; 2] = b": ";

/// The type of any established transport layer connection
pub type Stream = Box<dyn IO>;

/// Define a session identifier
pub trait UniqueID {
    fn id(&self) -> i32;
}

/// The abstraction of transport layer IO
pub trait IO:
    AsyncRead
    + AsyncWrite
    + UniqueID
    + Unpin
    + Debug
    + Send
    + Sync
{
    /// helper to cast as the reference of the concrete type
    fn as_any(&self) -> &dyn Any;
    /// helper to cast back of the concrete type
    fn into_any(self: Box<Self>) -> Box<dyn Any>;
}