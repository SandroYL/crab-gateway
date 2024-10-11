use std::{sync::Arc, time::{Duration, SystemTime}};

use tokio::{io::BufStream, net::TcpStream, time::Instant};

use crate::connections::{digest::{GetTimingDigest, TimingDigest}, row_connection::ProxyDigest};

enum RawStream {
    Tcp(TcpStream),
}

struct AccumulatedDuration {
    total: Duration,
    last_start: Option<Instant>,
}

// Large read buffering helps reducing syscalls with little trade-off
// Ssl layer always does "small" reads in 16k (TLS record size) so L4 read buffer helps a lot.
const BUF_READ_SIZE: usize = 64 * 1024;
// Small write buf to match MSS. Too large write buf delays real time communication.
// This buffering effectively implements something similar to Nagle's algorithm.
// The benefit is that user space can control when to flush, where Nagle's can't be controlled.
// And userspace buffering reduce both syscalls and small packets.
const BUF_WRITE_SIZE: usize = 1460;

pub struct Stream {
    stream: BufStream<RawStream>,
    buffer_write: bool,
    proxy_digest: Option<Arc<ProxyDigest>>,
    // when this connection is established
    pub established_ts: SystemTime,
    read_pending_time: AccumulatedDuration,
    write_pending_time: AccumulatedDuration,
}


impl GetTimingDigest for Stream {
    fn get_timing_digest(&self) -> Vec<Option<TimingDigest>> {
        let mut digest = Vec::with_capacity(2); //l4 TLS
        digest.push(Some(
            TimingDigest {
                established_ts: self.established_ts,
            }
        ));
        digest
    }
    
    fn get_read_pending_time(&self) -> std::time::Duration {
        self.read_pending_time.total
    }
    
    fn get_write_pending_time(&self) -> std::time::Duration {
       self.write_pending_time.total
    }
}