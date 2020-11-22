use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
    },
};

pub struct Driver {
    pub id: usize,
    pub future: &'static mut dyn Future<Output = !>,
}

impl Future for Driver {
    type Output = !;

    fn poll(self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe { self.map_unchecked_mut(|s| s.future) }.poll(ctx)
    }
}
