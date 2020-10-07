use crate::mem::Allocator;
use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

pub static mut POLL_CALL_COUNT: u16 = 0;

pub struct Driver {
    pub future: &'static mut dyn Future<Output = !>,
}

impl Driver {
    pub fn new(f: impl Future<Output = !> + 'static) -> Driver {
        let d = Driver {
            future: Allocator::get().new(f),
        };
        d
    }
}

impl Future for Driver {
    type Output = !;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            POLL_CALL_COUNT += 1;
        }
        unsafe { self.map_unchecked_mut(|s| s.future) }.poll(ctx)
    }
}
