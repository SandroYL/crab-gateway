use std::{future::Future, io::{self, IoSlice}, pin::Pin, task::{ready, Context, Poll}};
use bytes::Buf;
use tokio::io::AsyncWrite;

pub trait AsyncWriteVec {
    fn poll_write_vec<B: Buf>(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
        _buf: &mut B,
    ) -> Poll<io::Result<usize>>;

    fn write_vec<'a, B>(&'a mut self, src: &'a mut B) -> WriteVec<'a, Self, B>
    where
        Self: Sized,
        B: Buf,
    {
        WriteVec {
            writer: self,
            buf: src,
        }
    }

    fn write_vec_all<'a, B>(&'a mut self, src: &'a mut B) -> WriteVecAll<'a, Self, B>
    where 
        Self: Sized,
        B: Buf,
    {
        WriteVecAll {
            writer: self,
            buf: src,
        }
    }
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteVec<'a, W, B> {
    writer: &'a mut W,
    buf: &'a mut B,
}

#[must_use = "futures do nothing unless you `.await` or poll them"]
pub struct WriteVecAll<'a, W, B> {
    writer: &'a mut W,
    buf: &'a mut B,
}

impl<W, B> Future for WriteVecAll<'_, W, B> 
where
    W: AsyncWriteVec + Unpin,
    B: Buf,
{
    type Output = io::Result<()>;

    fn poll(mut self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Self::Output> {
        let me = &mut *self;
        while me.buf.has_remaining() {
            let n = ready!(Pin::new(&mut *me.writer).poll_write_vec(cx, me.buf))?;
            if n == 0 {
                return Poll::Ready(Err(io::ErrorKind::WriteZero.into()));
            }
        }
        Poll::Ready(Ok(()))
    }
}

impl<W, B> Future for WriteVec<'_, W, B>
where 
    W: AsyncWriteVec + Unpin,
    B: Buf,
{
    type Output = io::Result<usize>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let me = &mut *self;
        Pin::new(&mut *me.writer).poll_write_vec(cx, me.buf)
    }
}

/* from https://github.com/tokio-rs/tokio/blob/master/tokio-util/src/lib.rs#L177 */
impl<T> AsyncWriteVec for T
where
    T: AsyncWrite,
{
    fn poll_write_vec<B: Buf>(
        self: Pin<&mut Self>,
        ctx: &mut Context,
        buf: &mut B,
    ) -> Poll<io::Result<usize>> {
        const MAX_BUFS: usize = 64;

        if !buf.has_remaining() {
            return Poll::Ready(Ok(0));
        }

        let n = if self.is_write_vectored() {
            let mut slices = [IoSlice::new(&[]); MAX_BUFS];
            let cnt = buf.chunks_vectored(&mut slices);
            ready!(self.poll_write_vectored(ctx, &slices[..cnt]))?
        } else {
            ready!(self.poll_write(ctx, buf.chunk()))?
        };

        buf.advance(n);

        Poll::Ready(Ok(n))
    }
}