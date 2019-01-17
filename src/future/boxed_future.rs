use std::future::Future;
use std::pin::Pin;
use std::task::{LocalWaker, Poll};

pub(crate) struct BoxedFuture<'a, T>(Box<dyn Future<Output = T> + Send + 'a>);

impl<'a, T> BoxedFuture<'a, T> {
    pub fn new(v: Box<dyn Future<Output = T> + Send + 'a>) -> Self {
        BoxedFuture(v)
    }
}

impl<'a, T> Future for BoxedFuture<'a, T> {
    type Output = T;

    fn poll(self: Pin<&mut Self>, waker: &LocalWaker) -> Poll<T> {
        let me: &mut Self = self.get_mut();

        unsafe { Pin::new_unchecked(&mut *me.0).poll(waker) }
    }
}
