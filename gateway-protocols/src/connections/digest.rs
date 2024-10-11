use std::{sync::Arc, time::{Duration, SystemTime}};

use super::row_connection::ProxyDigest;


pub struct Digest {
    pub timing_digest: Vec<Option<TimingDigest>>,
    pub proxy_digest: Option<Arc<ProxyDigest>>,
}

pub struct TimingDigest {
    pub established_ts: SystemTime,
}

pub trait GetTimingDigest {
    fn get_timing_digest(&self) -> Vec<Option<TimingDigest>>;
    fn get_read_pending_time(&self) -> Duration {
        Duration::ZERO
    }    
    fn get_write_pending_time(&self) -> Duration {
        Duration::ZERO
    }
}


pub trait GetProxyDigest {
    fn get_proxy_digest(&self) -> Option<Arc<ProxyDigest>>;
    fn set_proxy_digest(&mut self, _digest: ProxyDigest){}
}