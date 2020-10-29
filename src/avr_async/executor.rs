use crate::avr_async::Driver;
use arduino_uno::hal::{
    clock::MHz16,
    port::mode::Floating,
    usart::Usart0,
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
use heapless::{
    consts::U8,
    Vec,
};
use ufmt::uwriteln;
use void::ResultVoidExt;

pub const MAX_DRIVERS: u8 = 8;
static mut EXECUTOR_INIT: bool = false;
pub struct Executor {
    drivers: Vec<Driver, U8>,
    work_queue: Vec<u8, U8>,
}

impl Executor {
    pub fn get() -> &'static mut Executor {
        static mut EXECUTOR: MaybeUninit<Executor> = MaybeUninit::uninit();
        unsafe {
            if !EXECUTOR_INIT {
                EXECUTOR.as_mut_ptr().write(Executor {
                    drivers: Vec::new(),
                    work_queue: Vec::new(),
                });
                EXECUTOR_INIT = true;
            }
            &mut *(EXECUTOR.as_mut_ptr() as *mut Executor)
        }
    }

    pub fn add_async_driver(&mut self, future: &'static mut dyn Future<Output = !>) {
        match self.drivers.push(Driver { future }) {
            Ok(_) => (),
            Err(_) => panic!("The `Executor.drivers` vector is full!"),
        }
    }

    #[inline(always)]
    pub fn add_work(&mut self, driver_id: u8) {
        // Until https://github.com/rust-lang/rust/issues/78260 is fixed,
        // these guards are necessary.  This is kinda brittle, if these
        // registers stop being used for the vector push things could
        // start breaking again.
        unsafe {
            llvm_asm!("push r26");
            llvm_asm!("push r27");
        }
        match self.work_queue.push(driver_id) {
            Ok(_) => (),
            Err(_) => panic!("The `Executor.work_queue` vector is full!"),
        }
        unsafe {
            llvm_asm!("pop r27");
            llvm_asm!("pop r26");
        }
    }

    pub fn run(&mut self, serial: &mut Usart0<MHz16, Floating>) {
        uwriteln!(serial, "executor is starting").void_unwrap();
        for driver_id in 0..self.drivers.len() {
            self.add_work(driver_id as u8);
        }
        loop {
            uwriteln!(serial, "executor woke up!").void_unwrap();
            if let Some(id) = self.work_queue.pop() {
                unsafe {
                    let task = Pin::new_unchecked(&mut self.drivers[id as usize]);
                    let waker = Waker::from_raw(RawWaker::new(&id as *const _ as *const _, &VTABLE));
                    let mut ctx = Context::from_waker(&waker);
                    task.poll(&mut ctx); // TODO handle Poll::Pending
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
    if e.work_queue.len() == 0 {
        let val = *(data as *const u8);
        e.add_work(val);
    }
}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}
