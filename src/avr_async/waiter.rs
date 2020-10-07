use crate::uno::{
    OCR0B,
    TCNT0,
};
use avr_hal_generic::avr_device;
use core::{
    future::Future,
    pin::Pin,
    task::{
        Context,
        Poll,
        Waker,
    },
};

pub struct Waiter {
    start_timer_value: u8,
    wait_ms: u32,
    interrupt_set: bool,
}
use avr_device::interrupt::free as critical_section;

pub static mut TIMER0_CMPB_ITERS: u32 = 0;
pub static mut TIMER0_CMPB_NEEDED_ITERS: u32 = 0;
pub static mut TIMER0_CMPB_REMAINDER: u8 = 0;
static mut CURRENT_WAKER: Option<Waker> = None;

impl Waiter {
    pub fn new(wait_ms: u32) -> Waiter {
        Waiter {
            start_timer_value: unsafe { *TCNT0 },
            wait_ms,
            interrupt_set: false,
        }
    }

    unsafe fn set_interrupt(mut self: Pin<&mut Self>) {
        critical_section(|_| {
            TIMER0_CMPB_ITERS = 0;
            TIMER0_CMPB_NEEDED_ITERS = self.wait_ms * 1000 / 1024;
            TIMER0_CMPB_REMAINDER = (self.wait_ms % 256) as u8;
            let TIMSK0 = 0x6E as *mut u8;
            *TIMSK0 |= 0x04; // Enable the TIMER0_COMPB interrupt
            *OCR0B = self.start_timer_value;
            self.interrupt_set = true;
        });
    }
}

impl Future for Waiter {
    type Output = ();

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context) -> Poll<Self::Output> {
        let TIMSK0 = 0x6E as *mut u8;
        if unsafe {
            *TIMSK0 & 0x04 != 00
                && (TIMER0_CMPB_ITERS > TIMER0_CMPB_NEEDED_ITERS
                    || (TIMER0_CMPB_ITERS == TIMER0_CMPB_NEEDED_ITERS && *TCNT0 > *OCR0B))
        } {
            unsafe {
                *TIMSK0 &= 0x03;
            }
            self.interrupt_set = false;
            return Poll::Ready(());
        } else if !self.interrupt_set {
            unsafe {
                CURRENT_WAKER = Some(ctx.waker().clone());
                self.set_interrupt();
            };
        }
        Poll::Pending
    }
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_COMPB() {
    TIMER0_CMPB_ITERS += 1;
    if TIMER0_CMPB_ITERS > TIMER0_CMPB_NEEDED_ITERS {
        if let Some(waker) = CURRENT_WAKER.take() {
            waker.wake();
        }
    }
}
