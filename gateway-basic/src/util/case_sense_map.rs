use std::collections::HashMap;
use gateway_error::Error;

pub struct CaseSenseMap<K, V> {
    inner: Result<HashMap<K, V>, Error>
}

impl<K, V> CaseSenseMap<K, V> {
    pub fn new() -> Self {
        Self {
            inner: Ok(HashMap::new())
        }
    }

    pub fn get() -> Some(V) {
        
    }
}