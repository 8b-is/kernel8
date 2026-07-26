//! A future that returns Pending exactly once, then Ready — the simplest
//! possible cooperative yield point, used only to prove the executor
//! actually interleaves tasks rather than running each to completion.
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct YieldNow(bool);

pub fn yield_now() -> YieldNow {
    YieldNow(false)
}

impl Future for YieldNow {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<()> {
        if self.0 {
            Poll::Ready(())
        } else {
            self.0 = true;
            Poll::Pending
        }
    }
}
