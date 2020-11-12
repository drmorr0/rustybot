use crate::{
    avr_async::Driver,
    mem::Allocator,
    uno::timers::{
        millis,
        WAITERS,
    },
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

pub const MAX_DRIVERS: usize = 8;
static mut EXECUTOR_INIT: bool = false;
pub struct Executor {
    drivers: [MaybeUninit<Driver>; MAX_DRIVERS],
    drivers_len: usize,
    work_queue: [usize; MAX_DRIVERS],
    work_queue_len: usize,
}

impl Executor {
    pub fn get() -> &'static mut Executor {
        static mut EXECUTOR: MaybeUninit<Executor> = MaybeUninit::uninit();
        unsafe {
            if !EXECUTOR_INIT {
                EXECUTOR.as_mut_ptr().write(Executor {
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
                    work_queue: [0; MAX_DRIVERS],
                    work_queue_len: 0,
                });
                EXECUTOR_INIT = true;
            }
            &mut *(EXECUTOR.as_mut_ptr() as *mut Executor)
        }
    }

    pub fn add_async_driver(&mut self, future: &'static mut dyn Future<Output = !>) {
        unsafe {
            self.drivers[self.drivers_len].as_mut_ptr().write(Driver { future });
        }
        self.drivers_len += 1;
    }

    pub fn add_work(&mut self, driver_id: usize) {
        if self.work_queue_len >= MAX_DRIVERS {
            panic!("The `Executor.work_queue` vector is full!");
        }
        self.work_queue[self.work_queue_len] = driver_id;
        self.work_queue_len += 1;
    }

    pub fn run(&mut self, serial: &mut Usart0<MHz16, Floating>) {
        uwriteln!(serial, "executor is starting").void_unwrap();
        for driver_id in 0..self.drivers_len {
            self.add_work(driver_id as usize);
        }
        loop {
            for i in 0..self.work_queue_len {
                let id = self.work_queue[i];
                uwriteln!(serial, "executor processing task {}!", id as u8).void_unwrap();
                unsafe {
                    let waker = Waker::from_raw(RawWaker::new(&id as *const _ as *const _, &VTABLE));
                    let mut ctx = Context::from_waker(&waker);
                    Pin::new_unchecked(self.drivers[id].assume_init_mut()).poll(&mut ctx);
                }
            }
            self.work_queue_len = 0;
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
