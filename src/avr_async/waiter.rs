use crate::uno::timers::{
    millis,
    register_timed_waker,
};
use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

pub struct Waiter {
    trigger_time_ms: u32,
    interrupt_set: bool,
}

impl Waiter {
    pub fn new(wait_ms: u32) -> Waiter {
        Waiter {
            trigger_time_ms: millis() + wait_ms,
            interrupt_set: false,
        }
    }
}

impl Future for Waiter {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        if millis() >= self.trigger_time_ms {
            self.interrupt_set = false;
            return Poll::Ready(());
        } else if !self.interrupt_set {
            register_timed_waker(self.trigger_time_ms, ctx.waker().clone());
            self.interrupt_set = true;
        }
        Poll::Pending
    }
}
