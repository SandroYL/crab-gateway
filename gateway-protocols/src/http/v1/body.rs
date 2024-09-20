
use bytes::{BufMut, BytesMut};
use log::{debug, trace, warn};
use tokio::io::AsyncRead;
use crate::{http::common::BODY_BUFFER_SIZE, util_code::buf_ref::BufRef};

use gateway_error::{error_trait::OrErr, Error, ErrorType, Result as Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseState {
    ToStart,
    Complete(ReadBytes),
    Partial(ReadBytes, RemainingBytes),
    Chunked(ReadBytes, ChunkSize, ChunkOffset, RemainingBytes),
    Done(ReadBytes),
    HTTP1_0(ReadBytes),
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

    pub async fn do_read_body<S> (&mut self, stream: &mut S, state: Option<ParseState>) -> Result<Option<BufRef>>
    where S: AsyncRead + Unpin + Send, 
    {
        use tokio::io::AsyncReadExt;
        
        let body_buf = self.body_buf.as_deref_mut().unwrap();
        let mut n = 0;
        std::mem::swap(&mut n, &mut self.rewind_buf_len);

        //如果没有需要回滚的data
        if n == 0 {
            n = stream
                .read(body_buf)
                .await
                .or_err(ErrorType::ReadError, "when reading body")?;
        }

        if state.is_some() {
            match (state.unwrap() == self.body_state, self.body_state) {
                (false, _) => Error::generate_error_with_root(
                    ErrorType::ReadError, 
                    &format!("wrong body state {:?}", self.body_state),
                    None),
                (true, ParseState::Partial(read, to_read)) => self.do_read_body_partial(n).await
                (true, ParseState::ToStart) => todo!(),
                (true, ParseState::Complete(_)) => todo!(),
                (true, ParseState::Chunked(_, _, _, _)) => todo!(),
                (true, ParseState::Done(_)) => todo!(),
                (true, ParseState::HTTP1_0(_)) => todo!(),
    
            }
        }



        Ok(None)
    }


    async fn do_read_body_partial(&mut self, n: usize) -> Result<Option<BufRef>> {
        match self.body_state {
            ParseState::Partial(read, to_read) => {
                debug!(
                    "BodyReader body_state: {:?}, 
                    read data from IO: {n}. ",
                    self.body_state
                );
                if n == 0 { //虽然有to_read, 但是读不了body, 证明连接断开了
                    self.body_state = PS::Done(read);
                    return Error::generate_error_with_root(ErrorType::ConnectionClosed, &format!("Peer permaturely closed connection with {} bytes of body remaining to read", to_read), None);
                } else if n >= to_read {
                    if n != to_read { //太多了 discard
                        warn!(
                            "Peer sent more data than expected: extra {}
                             bytes, discarded!",
                            n - to_read)
                    }
                    self.body_state = ParseState::Complete(read + to_read);
                    return Ok(Some(BufRef::new(0, to_read)));
                } 
                else {
                    self.body_state = ParseState::Partial(read + n, to_read);
                    return Ok(Some(BufRef::new(0, n)));
                }
            } 
            _ => Error::generate_error_with_root(ErrorType::ConnectProxyError, &format!("wrong body state {:?}", self.body_state), None),
        }
    }

    async fn do_read_body_http_1_0(&mut self, n: usize) -> Result<Option<BufRef>> {
        match self.body_state {
            ParseState::HTTP1_0(read) => {
                if n == 0 {
                    self.body_state = ParseState::Complete(read);
                    return Ok(None);
                } else {
                    self.body_state = ParseState::HTTP1_0(read + n);
                    return Ok(Some(BufRef::new(0, n)));
                }
            }
            _ => Error::generate_error_with_root(ErrorType::ConnectProxyError, &format!("wrong body state {:?}", self.body_state), None)
        }
    }

    async fn do_read_body_chunked<S> (&mut self, stream: &mut S , n: usize) -> Result<Option<BufRef>>
    where S: AsyncRead + Unpin + Send
    {
        use tokio::io::AsyncReadExt;
        match self.body_state {
            ParseState::Chunked(total_read, exist_buf_start, mut exist_buf_end, mut expect_from_io) => {
                if exist_buf_start == 0 {
                    let body_buf = self.body_buf.as_deref_mut().unwrap();
                    if exist_buf_end == 0 {
                        std::mem::swap(&mut exist_buf_end, &mut self.rewind_buf_len);
                        if exist_buf_end == 0 {
                            exist_buf_end = stream
                                .read(body_buf)
                                .await
                                .or_err(ErrorType::ReadError, "when reading body")?;
                        }
                    } else {
                        body_buf
                            .copy_within(exist_buf_end - expect_from_io..exist_buf_end, 0);
                        let new_bytes = stream
                            .read(&mut body_buf[expect_from_io..])
                            .await
                            .or_err(ErrorType::ReadError, "when reading body")?;
                        exist_buf_end + expect_from_io + new_bytes;
                        expect_from_io = 0;
                    }
                    self.body_state = self.body_state.new_buf(exist_buf_end);
                }

                if exist_buf_end == 0 {
                    self.body_state = self.body_state.done(0);
                    return Error::generate_error_with_root(ErrorType::ConnectionClosed, &format!(
                        "Connection prematurely closed without the termination chunk,
                         read {total_read} bytes"
                    ), None);
                } else {
                    if expect_from_io > 0 {
                        trace!(
                            "partial chunk payload, expecting_from_io: {}, 
                                existing_buf_end {}, buf: {:?} ",
                            expect_from_io,
                            exist_buf_end,
                            String::from_utf8_lossy(
                                &self.body_buf.as_ref().unwrap()[..exist_buf_end]
                            )
                        );
                        // 还没读完一个chunk
                        if expect_from_io >= exist_buf_end + 2 {
                            self.body_state = self.body_state.partial_chunk(
                                exist_buf_end,
                                expect_from_io - exist_buf_end, 
                            );
                            return Ok(Some(BufRef::new(0, exist_buf_end)));
                        }
                        /* EXPECTING DATA + CRLF OR JUST CRLF */
                        let payload_size = if expect_from_io > 2 {
                            expect_from_io - 2
                        } else {
                            0
                        };
                        if expect_from_io >= exist_buf_end {
                            self.body_state = self
                                .body_state
                                .partial_chunk(payload_size, expect_from_io - exist_buf_end);
                            return Ok(Some(BufRef(0, payload_size)));
                        }

                        self.body_state = self.body_state
                            .multi_chunk(payload_size, expect_from_io);
                        return Ok(Some(BufRef::new(0, payload_size)));
                    }
                    self.par
                }
            }
            _ => {}
        };
        return Error::generate_error_with_root(ErrorType::ConnectProxyError, &format!("wrong body state {:?}", self.body_state), None);
    }

    fn parse_chunked_buf (
        &mut self,
        buf_index_start: usize,
        buf_index_end: usize
    ) -> Result<Option<BufRef>> {
        let buf = &self.body_buf.as_ref().unwrap()[buf_index_start..buf_index_end];
        let chunk_status = httparse::parse_chunk_size(buf);
        match chunk_status {
            Ok(status) => {
                match status {
                    httparse::Status::Complete((payload_index, chunk_size)) => {
                        trace!(
                            "Got size {chunk_size}, payload_index: {payload_index},
                            chunk: {:?}",
                            String::from_utf8_lossy(buf)
                        );
                        let chunk_size = chunk_size as usize;
                        if chunk_size == 0 {
                            self.body_state = self.body_state.finish(0);
                            return Ok(None);
                        }
                        return Ok(None);
                    }
                    httparse::Status::Partial => {}
                }
            }
            Err(e) => {
                Error::generate_error_with_root(ErrorType::Custom("INVALID_CHUNK"), &format!("Invalid chucked encoding: {:?}", e), None)
            }
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

///以下是用来提高程序可读性的
type ReadBytes = usize;
type RemainingBytes = usize;
type ChunkSize = usize;
type ChunkOffset = usize;