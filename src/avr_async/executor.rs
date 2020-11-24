use crate::{
    avr_async::Driver,
    uno::Uno,
};
use arduino_uno::{
    hal::{
        clock::MHz16,
        port::mode::Floating,
        usart::Usart0,
    },
    prelude::*,
};
use core::{
    future::Future,
    mem::MaybeUninit,
    pin::Pin,
    task::{
        Context,
        Poll,
        RawWaker,
        RawWakerVTable,
        Waker,
    },
};
use micromath::F32Ext;
use ufmt::{
    uwrite,
    uwriteln,
};
use void::ResultVoidExt;

pub const NTASKS: usize = 8;
static mut EXECUTOR: Executor = Executor {
    drivers: [
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
        MaybeUninit::uninit(),
    ],
    drivers_len: 0,
    work_queue: [false; NTASKS],
};

pub struct Executor {
    drivers: [MaybeUninit<Driver>; NTASKS],
    drivers_len: usize,
    work_queue: [bool; NTASKS],
}

impl Executor {
    pub fn get() -> &'static mut Executor {
        unsafe { &mut EXECUTOR }
    }

    pub fn add_async_driver(&mut self, future: &'static mut dyn Future<Output = !>) {
        unsafe {
            self.drivers[self.drivers_len].as_mut_ptr().write(Driver {
                id: self.drivers_len,
                future,
            });
        }
        self.drivers_len += 1;
    }

    pub fn add_work(&mut self, driver_id: usize) {
        self.work_queue[driver_id] = true;
    }

    pub fn run(&mut self, uno: &mut Uno) {
        //uwriteln!(uno.serial, "executor is starting").void_unwrap();
        for driver_id in 0..self.drivers_len {
            self.add_work(driver_id as usize);
        }
        loop {
            //uno.write_state();
            for id in 0..self.drivers_len {
                if !self.work_queue[id] {
                    continue;
                }
                //uwriteln!(uno.serial, "executor processing task {}!", id as u8).void_unwrap();
                unsafe {
                    self.work_queue[id] = false;
                    // The drivers are part of a static object, so we know they won't move; thus
                    // it's safe to pin them
                    let driver = Pin::new_unchecked(self.drivers[id].assume_init_mut());
                    let waker = Waker::from_raw(RawWaker::new(&driver.id as *const _ as *const _, &VTABLE));
                    let mut ctx = Context::from_waker(&waker);

                    // the drivers are infinite loops, so they will never return Poll::Ready
                    // we just discard the result to silence the compiler warning
                    let _ = driver.poll(&mut ctx);
                }
            }
            unsafe {
                llvm_asm!("sleep");
            }
        }
    }
}

static VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(data: *const ()) -> RawWaker {
    RawWaker::new(data, &VTABLE)
}

unsafe fn wake(data: *const ()) {
    let e = Executor::get();
    let val = *(data as *const usize);
    e.add_work(val);
}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}
