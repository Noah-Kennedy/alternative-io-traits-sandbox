use pin_project::pin_project;
use std::future::Future;
use std::io;
use std::marker::PhantomPinned;
use std::pin::{pin, Pin};
use std::task::{Context, Poll};

pub trait OwnedBufferRead {
    type Buffer: AsMut<[u8]>;
    fn poll_read(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<Self::Buffer>>;
}

pub trait OwnedBufferReadExt {
    fn read(&mut self) -> Read<Self> {
        Read {
            io: self,
            _pin: PhantomPinned::default(),
        }
    }
}

#[pin_project]
pub struct Read<'a, I: ?Sized> {
    io: &'a mut I,
    #[pin]
    _pin: PhantomPinned,
}

impl<'a, I> Future for Read<'a, I>
where
    I: OwnedBufferRead + Unpin + ?Sized,
{
    type Output = io::Result<I::Buffer>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let projected = self.project();
        Pin::new(&mut **projected.io).poll_recv(cx)
    }
}

impl<I: OwnedBufferRead> OwnedBufferReadExt for I {}
