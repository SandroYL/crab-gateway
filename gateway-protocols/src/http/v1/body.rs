
use bytes::{BufMut, BytesMut};
use log::{debug, trace, warn};
use crate::{http::common::{BODY_BUFFER_SIZE, PARTIAL_CHUNK_HHEAD_LIMIT}, util_code::buf_ref::BufRef};
use tokio::io::{AsyncRead, AsyncReadExt};
use gateway_error::{error_trait::OrErr, Error, ErrorType, Result as Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseState {
    ToStart,
    Complete(ReadBytes),
    Partial(ReadBytes, RemainingBytes),
    Chunked(ReadBytes, NxtChunkStartIdx, NxtChunkEndIdx, RemainingBytes),
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

    pub async fn do_read_body<S> (&mut self, stream: &mut S) -> Result<Option<BufRef>>
    where S: AsyncRead + Unpin + Send, 
    {
        match (self.body_state) {
            ParseState::Partial(_, _) => self.do_read_body_partial(stream).await,
            ParseState::ToStart => Ok(None),
            ParseState::Complete(_) => Ok(None),
            ParseState::Chunked(_, _, _, _) => self.do_read_body_chunked(stream).await,
            ParseState::Done(_) => Ok(None),
            ParseState::HTTP1_0(_) => self.do_read_body_http_1_0(stream).await,
        }
    }


    async fn do_read_body_partial<S>(&mut self, stream: &mut S) -> Result<Option<BufRef>> 
    where S: AsyncRead + Unpin + Send,
    {
        let mut body_buf = self.body_buf.as_deref_mut().unwrap();
        //如果没有需要回滚的data
        let mut n = 0;
        std::mem::swap(&mut n, &mut self.rewind_buf_len);
        if n == 0 {
            n = stream
                .read(&mut body_buf)
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
                    self.body_state = ParseState::Partial(read + n, to_read - n);
                    return Ok(Some(BufRef::new(0, n)));
                }
            } 
            _ => Error::generate_error_with_root(ErrorType::ConnectProxyError, &format!("wrong body state {:?}", self.body_state), None),
        }
    }

    async fn do_read_body_http_1_0<S>(&mut self, stream: &mut S) -> Result<Option<BufRef>>
    where S: AsyncRead + Unpin + Send,
    {
        let body_buf = self.body_buf.as_deref_mut().unwrap();
        //如果没有需要回滚的data
        let mut n = 0;
        std::mem::swap(&mut n, &mut self.rewind_buf_len);
        if n == 0 {
            n = stream
                .read(body_buf)
                .await
                .or_err(ErrorType::ReadError, "when reading body")?;
        }
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

    async fn do_read_body_chunked<S> (&mut self, stream: &mut S) -> Result<Option<BufRef>>
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
                        exist_buf_end = expect_from_io + new_bytes;
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
                } else if expect_from_io > 0 {
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
                self.parse_chunked_buf(exist_buf_start, exist_buf_end)
            }
            _ => Error::generate_error_with_root(ErrorType::ConnectProxyError, &format!("wrong body state {:?}", self.body_state), None)
        }
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
                            "Got size {chunk_size}, payload_index: {payload_index},chunk: {:?}",
                            String::from_utf8_lossy(buf)
                        );
                        let chunk_size = chunk_size as usize;
                        if chunk_size == 0 {
                            self.body_state = self.body_state.finish(0);
                            return Ok(None);
                        }

                        let data_end_index = payload_index + chunk_size;
                        let chunk_end_index = data_end_index + 2;
                        if chunk_end_index >= buf.len() {
                            let actual_size = if data_end_index > buf.len() {
                                buf.len() - payload_index
                            } else {
                                chunk_size
                            };
                            self.body_state = self
                                .body_state
                                .partial_chunk(actual_size, chunk_end_index - buf.len());
                            return Ok(Some(BufRef::new(
                                buf_index_start + payload_index, actual_size)));
                        }
                        self.body_state = self
                            .body_state
                            .multi_chunk(chunk_size, buf_index_start + chunk_end_index);

                        return Ok(Some(BufRef::new(
                            buf_index_start + payload_index, chunk_size)));
                    }
                    httparse::Status::Partial => {
                        if buf.len() > PARTIAL_CHUNK_HHEAD_LIMIT {
                            self.body_state = self.body_state.done(0);
                             Error::generate_error_with_root(ErrorType::Custom("INVALID_CHUNK"), "Chunk extover limit", None)
                        } else {
                            self.body_state = self.body_state.partial_chunk_head(buf_index_end, buf.len());
                            Ok(Some(BufRef::new(0, 0)))
                        }
                    }
                }
            }
            Err(e) => {
                Error::generate_error_with_root(ErrorType::Custom("INVALID_CHUNK"), &format!("Invalid chucked encoding: {:?}", e), None)
            }
        }
    }
}

///以下是用来提高程序可读性的
type ReadBytes = usize;
type RemainingBytes = usize;
type NxtChunkStartIdx = usize;
type NxtChunkEndIdx = usize;


///
/// unit test read body!
#[cfg(test)]
mod partial_test {
    use tokio_test::io::Builder;

    use crate::connections::row_connection::ConnectProxyError;

    use super::*;

    fn init_log() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    #[tokio::test]
    async fn read_partial_with_fixed_length() {
        init_log();
        /* Perfectly fixed */
        let input = b"abcde";
        let mut mock_io = Builder::new().read(&input[..]).build();
        let mut body_reader = BodyReader::new();
        body_reader.init_content_length(5, b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();

        assert_eq!(res, BufRef::new(0, 5));
        assert_eq!(body_reader.body_state, ParseState::Complete(5));
        assert_eq!(input, body_reader.get_body(&res));

        /* get stream data too long */
        let input = b"asdasdasd";
        mock_io = Builder::new().read(&input[..]).build();
        body_reader = BodyReader::new();
        body_reader.init_content_length(5, b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();

        assert_eq!(res, BufRef::new(0, 5));
        assert_eq!(body_reader.body_state, ParseState::Complete(5));
        assert_eq!(b"asdas", body_reader.get_body(&res));

        /*  not enough */
        let input = b"asdasdasd";
        mock_io = Builder::new().read(&input[..]).build();
        body_reader = BodyReader::new();
        body_reader.init_content_length(10, b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();

        assert_eq!(res, BufRef::new(0, 9));
        assert_eq!(body_reader.body_state, ParseState::Partial(9, 1));
        assert_eq!(b"asdasdasd", body_reader.get_body(&res));

        /* partial transport */
        let input1 = b"abc";
        let input2 = b"abc";
        mock_io = Builder::new().read(&input1[..]).read(&input2[..]).build();
        body_reader = BodyReader::new();
        body_reader.init_content_length(6, b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::Partial(3, 3));
        assert_eq!(input1, body_reader.get_body(&res));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::Complete(6));
        assert_eq!(input2, body_reader.get_body(&res));
    }

    #[tokio::test]
    async fn read_partial_with_multi_packets() {
        init_log();
        let input1 = b"abc";
        let input2 = b"abc";
        let mut mock_io = Builder::new().read(&input1[..]).read(&input2[..]).build();
        let mut body_reader = BodyReader::new();
        body_reader.init_content_length(6, b"");
        assert_eq!(body_reader.body_state, ParseState::Partial(0, 6));
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::Partial(3, 3));
        assert_eq!(input1, body_reader.get_body(&res));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::Complete(6));
        assert_eq!(input2, body_reader.get_body(&res));
    }

    #[tokio::test]
    async fn read_partial_disconnnect() {
        let input1 = b"abc";
        let input2 = b"";
        let mut mock_io = Builder::new().read(&input1[..]).read(&input2[..]).build();
        let mut body_reader = BodyReader::new();
        body_reader.init_content_length(9, b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::Partial(3, 6));
        assert_eq!(input1, body_reader.get_body(&res));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap_err();
        assert_eq!(&ErrorType::ConnectionClosed, res.etype());
        assert_eq!(body_reader.body_state, ParseState::Done(3));
    }

    #[tokio::test]
    async fn read_with_rewind() {
        let rewind = b"ab";
        let input = b"abc";
        let mut mock_io = Builder::new().read(&input[..]).build();
        let mut body_reader = BodyReader::new();
        body_reader.init_content_length(5, rewind);
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 2));
        assert_eq!(body_reader.body_state, ParseState::Partial(2, 3));
        assert_eq!(rewind, body_reader.get_body(&res));
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::Complete(5));
        assert_eq!(input, body_reader.get_body(&res));
    }

    #[tokio::test]
    async fn read_with_body_http10() {
        init_log();
        let inputs = ["abc", "efg", "123", ""];
        let mut mock_io = Builder::new();
        for input in inputs {
            mock_io.read(input.as_bytes());
        }
        let mut mock_io = mock_io.build();
        let mut body_reader = BodyReader::new();
        body_reader.init_http10(b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::HTTP1_0(3));
        assert_eq!(inputs[0].as_bytes(), body_reader.get_body(&res));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::HTTP1_0(6));
        assert_eq!(inputs[1].as_bytes(), body_reader.get_body(&res));
        
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::HTTP1_0(9));
        assert_eq!(inputs[2].as_bytes(), body_reader.get_body(&res));
        
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(9));
    }

    #[tokio::test]
    async fn read_with_body_http10_rewind() {
        init_log();
        let rewind = b"cmd";
        let inputs = ["abc", ""];
        let mut mock_io = Builder::new();
        for input in inputs {
            mock_io.read(input.as_bytes());
        }
        let mut mock_io = mock_io.build();
        let mut body_reader = BodyReader::new();
        body_reader.init_http10(rewind);
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::HTTP1_0(3));
        assert_eq!(rewind, body_reader.get_body(&res));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 3));
        assert_eq!(body_reader.body_state, ParseState::HTTP1_0(6));
        assert_eq!(inputs[0].as_bytes(), body_reader.get_body(&res));
    
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(6));
    }

    #[tokio::test]
    async fn read_with_zero_chunk() {
        init_log();
        let input = b"0\r\n\r\n";
        let mut mock_io = Builder::new().read(&input[..]).build();
        let mut body_reader = BodyReader::new();
        body_reader.init_chunked(b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(0));
    }

    #[tokio::test]
    async fn read_with_chunk_ext() {
        init_log();
        let input = b"0;aaaa\r\n\r\n";
        let mut mock_io = Builder::new().read(&input[..]).build();
        let mut body_reader = BodyReader::new();
        body_reader.init_chunked(b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(0));
    }

    #[tokio::test]
    async fn read_with_chunk_fixed() {
        init_log();
        let input1 = b"1\r\na\r\n";
        let input2 = b"0\r\n\r\n";
        let mut mock_io = Builder::new()
            .read(&input1[..])
            .read(&input2[..])
            .build();
        let mut body_reader = BodyReader::new();
        body_reader.init_chunked(b"");
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 1));
        assert_eq!(&input1[3..4], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(1, 0, 0, 0));
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(1));
    }

    #[tokio::test]
    async fn read_with_chunk_multi_fixed_rewind() {
        init_log();
        let rewind = b"9\r\n123456789\r\n";
        let input1 = b"5\r\nabcde\r\n";
        let input2 = b"3\r\nfg";
        let input2_1 = b"h\r\n";
        let input3 = b"0\r\n\r\n";
        let mut mock_io = Builder::new()
            .read(&input1[..])
            .read(&input2[..])
            .read(&input2_1[..])
            .read(&input3[..])
            .build();
        let mut body_reader = BodyReader::new();
        body_reader.init_chunked(rewind);
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 9));
        assert_eq!(&rewind[3..12], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(9, 0, 0, 0));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 5));
        assert_eq!(&input1[3..8], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(14, 0, 0, 0));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 2));
        assert_eq!(&input2[3..5], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(16, 0, 0, 3));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 1));
        assert_eq!(&input2_1[0..1], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(17, 0, 0, 0));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(17));
    }

    #[tokio::test]
    async fn read_with_chunk_multi_inone_read() {
        init_log();
        let start_input = b"c\r";
        let start_input_1 = b"\n";
        let input1 = b"1\r\na\r\n2\r\nbc\r\n";
        let input2 = b"3\r\n123\r\n3\r\nefg\r\n3\r\nhi";
        let input2_1 = b"j\r\n";
        let input3 = b"0\r\n\r\n";
        let mut mock_io = Builder::new()
            .read(start_input)
            .read(start_input_1)
            .read(input1)
            .read(input2)
            .read(input2_1)
            .read(input3)
            .build();
        let mut body_reader = BodyReader::new();
        body_reader.init_chunked(b"3\r\nab");

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 2));
        assert_eq!(b"ab", body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(2, 0, 0, 3));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 1));
        assert_eq!(&start_input[0..1], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(3, 0, 0, 1));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef(0, 0));
        assert_eq!(body_reader.body_state, ParseState::Chunked(3, 0, 0, 0));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 1));
        assert_eq!(&input1[3..4], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(4, 6, 13, 0));
        
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(9, 2));
        assert_eq!(&input1[9..11], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(6, 0, 0, 0));
    
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(3, 3));
        assert_eq!(&input2[3..6], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(9, 8, 21, 0));
    
        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(11, 3));
        assert_eq!(&input2[11..14], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(12, 16, 21, 0));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(19, 2));
        assert_eq!(&input2[19..21], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(14, 0, 0, 3));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap().unwrap();
        assert_eq!(res, BufRef::new(0, 1));
        assert_eq!(&input2_1[0..1], body_reader.get_body(&res));
        assert_eq!(body_reader.body_state, ParseState::Chunked(15, 0, 0, 0));

        let res = body_reader.do_read_body(&mut mock_io).await.unwrap();
        assert_eq!(res, None);
        assert_eq!(body_reader.body_state, ParseState::Complete(15));
    }
} 