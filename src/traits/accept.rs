use std::pin::Pin;
use std::task::{Context, Poll};

pub trait Accept {
    type Conn;
    type Error;

    fn poll_accept(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Self::Conn, Self::Error>>>;
}
