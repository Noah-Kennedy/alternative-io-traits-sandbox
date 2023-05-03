//! I don't like this, but I can't think of anything better.

use pin_project::pin_project;
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::pin::Pin;
use std::task::{Context, Poll};

pub trait OwnedBufferWrite {
    type Buffer: AsMut<[u8]>;
    /// Poll to see if a wrote can be started.
    ///
    /// Writes do not complete just because this returns Poll::Ready().
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut Option<Self::Buffer>,
    ) -> Poll<io::Result<()>>;

    /// This is actually an important method, unlike with the tokio traits.
    /// todo flush key to allow for flushing at a specific point in the pipeline?
    /// todo implement flush future, its trivial so im ignoring it for now
    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>>;
}

pub trait OwnedBufferWriteExt: OwnedBufferWrite {
    fn write(&mut self, buf: Self::Buffer) -> Write<Self> {
        Write {
            io: &mut self,
            buffer: Some(buf),
            _pin: Default::default(),
        }
    }
}

#[pin_project]
pub struct Write<'a, I: ?Sized>
where
    I: OwnedBufferWrite,
    I::Buffer: 'a,
{
    io: &'a mut I,
    buffer: Option<I::Buffer>,
    #[pin]
    _pin: PhantomPinned,
}

impl<'a, I> Future for Write<'a, I> {
    type Output = io::Result<()>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        todo!()
    }
}
