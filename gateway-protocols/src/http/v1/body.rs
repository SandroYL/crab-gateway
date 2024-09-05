use bytes::BytesMut;

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