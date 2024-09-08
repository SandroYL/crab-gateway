use std::num::ParseFloatError;

use bytes::{BufMut, Bytes, BytesMut};
use log::{debug, warn};
use tokio::io::AsyncRead;

use crate::{http::common::BODY_BUFFER_SIZE, util_code::buf_ref::{self, BufRef}};

use gateway_error::{error_trait::OrErr, Error, ErrorType, Result as Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseState {
    ToStart,
    Complete(usize),                     // total size
    Partial(usize, usize),               // size read, remaining size
    Chunked(usize, usize, usize, usize), // size read, next to read in current buf start, read in current buf start, remaining chucked size to read from IO
    Done(usize),                         // done but there is error, size read
    HTTP1_0(usize),                      // read until connection closed, size read
}

type PS = ParseState;

impl ParseState {
    pub fn finish(&self, additional_bytes: usize) -> Self {
        match self {
            PS::Partial(read, to_read) => PS::Complete(read + to_read),
            PS::Chunked(read, _, _, _) => PS::Complete(read + additional_bytes),
            PS::HTTP1_0(read) => PS::Complete(read + additional_bytes),
            _ => self.clone(), /* invalid transaction */
        }
    }

    pub fn done(&self, additional_bytes: usize) -> Self {
        match self {
            PS::Partial(read, _) => PS::Done(read + additional_bytes),
            PS::Chunked(read, _, _, _) => PS::Done(read + additional_bytes),
            PS::HTTP1_0(read) => PS::Done(read + additional_bytes),
            _ => self.clone(), /* invalid transaction */
        }
    }

    pub fn partial_chunk(&self, bytes_read: usize, bytes_to_read: usize) -> Self {
        match self {
            PS::Chunked(read, _, _, _) => PS::Chunked(read + bytes_read, 0, 0, bytes_to_read),
            _ => self.clone(), /* invalid transaction */
        }
    }

    pub fn multi_chunk(&self, bytes_read: usize, buf_start_index: usize) -> Self {
        match self {
            PS::Chunked(read, _, buf_end, _) => {
                PS::Chunked(read + bytes_read, buf_start_index, *buf_end, 0)
            }
            _ => self.clone(), /* invalid transaction */
        }
    }

    pub fn partial_chunk_head(&self, head_end: usize, head_size: usize) -> Self {
        match self {
            /* inform reader to read more to form a legal chunk */
            PS::Chunked(read, _, _, _) => PS::Chunked(*read, 0, head_end, head_size),
            _ => self.clone(), /* invalid transaction */
        }
    }

    pub fn new_buf(&self, buf_end: usize) -> Self {
        match self {
            PS::Chunked(read, _, _, _) => PS::Chunked(*read, 0, buf_end, 0),
            _ => self.clone(), /* invalid transaction */
        }
    }
}

pub struct BodyReader {
    pub body_state: ParseState,
    pub body_buf: Option<BytesMut>,
    pub body_buf_size: usize,
    rewind_buf_len: usize,
}

impl BodyReader {
    pub fn new() -> Self {
        BodyReader {
            body_state: ParseState::ToStart,
            body_buf: None,
            body_buf_size: BODY_BUFFER_SIZE,
            rewind_buf_len: 0,
        }
    }

    pub fn need_init(&self) -> bool {
        self.body_state == ParseState::ToStart
    }

    pub fn reinit(&mut self) {
        self.body_state = ParseState::ToStart;
    }

    pub fn body_done(&self) -> bool {
        matches!(self.body_state, ParseState::Complete(_) | ParseState::Done(_))
    }

    fn prepare_buf(&mut self, buf_to_rewind: &[u8]) {
        let mut body_buf = BytesMut::with_capacity(self.body_buf_size);
        if !buf_to_rewind.is_empty() {
            self.rewind_buf_len = buf_to_rewind.len();
            body_buf.put_slice(buf_to_rewind);
        }
        if self.body_buf_size > buf_to_rewind.len() {
            unsafe {
                body_buf.set_len(self.body_buf_size);
            }
        }
        self.body_buf = Some(body_buf);
    }

    pub fn init_chunked(&mut self, buf_to_rewind: &[u8]) {
        self.body_state = ParseState::Chunked(0, 0, 0, 0);
        self.prepare_buf(buf_to_rewind);
    }

    pub fn init_content_length(&mut self, cl: usize, buf_to_rewind: &[u8]) {
        match cl {
            0 => self.body_state = ParseState::Complete(0),
            _ => {
                self.prepare_buf(buf_to_rewind);
                self.body_state = PS::Partial(0, cl)
            }
        }
    }

    pub fn init_http10(&mut self, buf_to_rewind: &[u8]) {
        self.prepare_buf(buf_to_rewind);
        self.body_state = ParseState::HTTP1_0(0);
    }

    pub fn get_body(&self, buf_ref: &BufRef) -> &[u8] {
        buf_ref.get(self.body_buf.as_ref().unwrap())
    }

    pub fn body_empty(&self) -> bool {
        self.body_state == PS::Complete(0)
    }

    pub async fn do_read_body<S>(&mut self, stream: &mut S) -> Result<Option<BufRef>>
    where S: AsyncRead + Unpin + Send,
    {
        use tokio::io::AsyncReadExt;

        let body_buf = self.body_buf.as_deref_mut().unwrap();
        let mut n = self.rewind_buf_len;
        self.rewind_buf_len = 0;

        if n == 0 {
            n = stream
                .read(body_buf)
                .await
                .or_err(ErrorType::ReadError, "when reading body")?;
        }

        match self.body_state {
            ParseState::Partial(read, to_read) => {
                debug!(
                    "BodyReader body_state: {:?}, 
                    read data from IO: {n}. ",
                    self.body_state
                );
                if n == 0 {
                    self.body_state = PS::Done(read);
                    return Error::generate_error_with_root(ErrorType::ConnectionClosed, &format!("Peer permaturely closed connection with {} bytes of body remaining to read", to_read), None);
                } else if n >= to_read {
                    if n != to_read {
                        warn!(
                            "Peer sent more data than expected: extra {}
                             bytes, discarded!",
                            n - to_read)
                    }
                    self.body_state = ParseState::Complete(read + to_read);
                    return Ok(Some(BufRef::new(0, to_read)));
                } else {
                    self.body_state = ParseState::Partial(read + n, to_read);
                    return Ok(Some(BufRef::new(0, n)));
                }
            } 
            _ => panic!("wrong body state: {:?}", self.body_state),

        }
    }

    pub async fn read_body<S> (&mut self, stream: &mut S) -> Result<Option<BufRef>>
    where
        S: AsyncRead + Unpin + Send, 
    {
        match self.body_state {
            ParseState::Complete(_) => Ok(None),
            ParseState::Done(_) => Ok(None),
            ParseState::Partial(_, _) => Ok(None),
            ParseState::Chunked(_, _, _, _) => Ok(None),
            ParseState::HTTP1_0(_) => Ok(None),
            ParseState::ToStart => panic!("not init BodyReader"),
        }
    }

}