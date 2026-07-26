//! Cooperative multitasking via async/await. Adapted from the real blog_os
//! reference (os.phil-opp.com/async-await) — deliberately NOT preemptive
//! multi-process scheduling with hand-written context-switch assembly,
//! which is a separate, much bigger undertaking even that tutorial defers.
//! A task here is a heap-allocated, pinned future that yields () and is
//! polled cooperatively — it must yield control back voluntarily, there's
//! no timer-driven preemption.
pub mod simple_executor;
pub mod yield_now;

use alloc::boxed::Box;
use core::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};

pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static) -> Task {
        Task { future: Box::pin(future) }
    }

    fn poll(&mut self, context: &mut Context) -> Poll<()> {
        self.future.as_mut().poll(context)
    }
}
