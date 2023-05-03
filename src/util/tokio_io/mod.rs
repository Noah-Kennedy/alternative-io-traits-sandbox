use crate::{Accept, OwnedBufferRead};
use pin_project::pin_project;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{ready, Context, Poll};
use std::{io, mem};
use tokio::io::{AsyncRead, ReadBuf};
use tokio::net::{TcpListener, TcpStream};

pub struct TokioAcceptor {
    pool: BufferPool,
    listener: TcpListener,
}

#[pin_project]
pub struct TokioStream {
    #[pin]
    tcp: TcpStream,
    pool: BufferPool,
}

impl Accept for TokioAcceptor {
    type Conn = TokioStream;
    type Error = io::Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>> {
        let this = self.get_mut();

        let (stream, _addr) = ready!(this.listener.poll_accept(cx))?;

        Poll::Ready(Some(Ok(TokioStream {
            tcp: stream,
            pool: this.pool.clone(),
        })))
    }
}

impl OwnedBufferRead for TokioStream {
    type Buffer = PoolBuf;

    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Self::Buffer>> {
        ready!(self.tcp.poll_read_ready(cx))?;

        let mut buffer = self.pool.take();

        let mut read_buf = ReadBuf::new(buffer.as_mut());

        let projected = self.project();

        ready!(projected.tcp.poll_read(cx, &mut read_buf))?;

        let n = read_buf.filled().len();

        buffer.buf.truncate(n);

        Poll::Ready(Ok(buffer))
    }
}

#[derive(Clone)]
struct BufferPool {
    core: Arc<BufferPoolCore>,
}

struct BufferPoolCore {
    buffers: Mutex<Vec<Vec<u8>>>,
    max_pool_size: usize,
    buffer_size: usize,
}

pub struct PoolBuf {
    buf: Vec<u8>,
    pool: Arc<BufferPoolCore>,
}

impl BufferPool {
    fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        Self {
            core: Arc::new(BufferPoolCore {
                buffers: Mutex::new(vec![]),
                max_pool_size,
                buffer_size,
            }),
        }
    }

    fn take(&self) -> PoolBuf {
        let mut guard = self.core.buffers.lock().unwrap();

        if let Some(buf) = guard.pop() {
            PoolBuf {
                pool: self.core.clone(),
                buf,
            }
        } else {
            PoolBuf {
                pool: self.core.clone(),
                buf: vec![0; self.core.buffer_size],
            }
        }
    }
}

impl AsMut<[u8]> for PoolBuf {
    fn as_mut(&mut self) -> &mut [u8] {
        self.buf.as_mut()
    }
}

impl Drop for PoolBuf {
    fn drop(&mut self) {
        if let Ok(mut guard) = self.pool.buffers.lock() {
            // not a fan of doing this here, as it means we can churn more than id like, but its
            // either do the check here or apply backpressure at point of buffer generation, and
            // risk the system locking up
            if guard.len() < self.pool.max_pool_size {
                self.buf.resize(self.pool.buffer_size, 0);
                let tmp = Vec::new();
                guard.push(mem::replace(&mut self.buf, tmp));
            }
        }
    }
}
