use crate::{
    avr_async::{
        driver::*,
        Driver,
    },
    uno::{
        led::TOGGLE_COUNT,
        timers::micros,
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
use heapless::{
    consts::U8,
    Vec,
};
use micromath::F32Ext;
use nb;
use ufmt::{
    uwrite,
    uwriteln,
};

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

    pub fn add_driver(&mut self, driver: Driver) {
        self.drivers.push(driver);
    }

    #[inline(always)]
    pub fn add_work(&mut self, driver_id: u8) {
        unsafe {
            llvm_asm!("push r26");
            llvm_asm!("push r27");
        }
        self.work_queue.push(driver_id);
        unsafe {
            llvm_asm!("pop r27");
            llvm_asm!("pop r26");
        }
    }

    pub fn run(&mut self, serial: &mut Usart0<MHz16, Floating>) {
        uwriteln!(serial, "executor is starting");
        for driver_id in 0..self.drivers.len() {
            self.add_work(driver_id as u8);
        }
        loop {
            let now = micros();
            let upper_padding = 5 - ((((now >> 16) as u16) as f32).log10() as u16);
            let lower_padding = 5 - (((now as u16) as f32).log10() as u16);
            for _ in 0..upper_padding {
                nb::block!(serial.write('0' as u8));
            }
            uwrite!(serial, "{}", (now >> 16) as u16);
            for _ in 0..lower_padding {
                nb::block!(serial.write('0' as u8));
            }
            uwrite!(serial, "{} ", now as u16);

            uwriteln!(serial, "executor woke up!");
            if let Some(id) = self.work_queue.pop() {
                unsafe {
                    let task = Pin::new_unchecked(&mut self.drivers[id as usize]);
                    let waker = Waker::from_raw(RawWaker::new(&id as *const _ as *const _, &VTABLE));
                    let mut ctx = Context::from_waker(&waker);
                    task.poll(&mut ctx);
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
