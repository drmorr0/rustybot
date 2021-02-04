use crate::util::*;
use arduino_uno::pac::TC0 as Timer0;
use avr_hal_generic::{
    avr_device,
    avr_device::interrupt::free as critical_section,
};
use core::{
    ptr::read_volatile,
    task::Waker,
};
use minarray::MinArray;

static mut TIMER0_OVF_COUNT: u32 = 0;
static mut ELAPSED_MS: u32 = 0;

// Using prescale_64 gives 64 / 16000000 = 4us per tick;
// The timer overflows every 4 * 256 = 1024us
const TIMER0_TICK_US: u32 = 4;
const TIMER0_OVF_US: u32 = 1024;
const TIMER0_TICKS_PER_MS: u8 = 250; // 1000us / (4us per tick) = 250 ticks/ms

static mut WAITERS: MinArray<Waker> = MinArray::new();

pub fn init_timers(t0: &Timer0) {
    t0.tccr0b.write(|w| w.cs0().prescale_64());
    t0.tcnt0.write(|w| unsafe { w.bits(0) });
    t0.timsk0.write(|w| unsafe { w.bits(0x03) }); // enable overflow interrupt and COMPA interrupt
    t0.ocr0a.write(|w| unsafe { w.bits(TIMER0_TICKS_PER_MS) });
}

// In *theory* this wouldn't overflow for (255 * 1024 + 4294967295 * 1024)us, but since
// it can only return a 32-bit integer, it actually wraps around after about 71 minutes.
//
// Bummer.
pub fn micros() -> u32 {
    critical_section(|_| micros_no_interrupt())
}

// Call this function if you're already in a disabled-interrupt context to avoid unnecessary
// sei/cli (interrupt enable/disable) instructions
pub fn micros_no_interrupt() -> u32 {
    unsafe {
        // If the TIMER0_OVF interrupt fires and interrupts are disabled, the TCNT0 register will
        // still overflow but the value in the TIMER0_OVF_COUNT will be incorrect, leading to this
        // function returning incorrect values.  We still could get incorrect values if the
        // interrupt fires between reading TIFR0 and TCNT0, but this (seems to) happen rarely.  In
        // this case we will add an extra 1024us into the timer, which will be rectified the next
        // time the function is called.
        let count0 = read_volatile(TCNT0) as u32;
        let extra_ovf = (read_volatile(TIFR0) & 1) as u32;
        (count0 as u32) * TIMER0_TICK_US + (TIMER0_OVF_COUNT + extra_ovf) * TIMER0_OVF_US
    }
}

// This will overflow after about 49 days
pub fn millis() -> u32 {
    critical_section(|_| unsafe { ELAPSED_MS })
}

pub fn register_timed_waker(trigger_time_ms: u32, waker: Waker) {
    critical_section(|_| unsafe {
        WAITERS.push(trigger_time_ms, waker);
    });
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_OVF() {
    TIMER0_OVF_COUNT += 1;
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_COMPA() {
    ELAPSED_MS += 1;

    if ELAPSED_MS > WAITERS.min {
        for (_, waker) in WAITERS.take_less_than(ELAPSED_MS) {
            waker.wake();
        }
    }

    *OCR0A += TIMER0_TICKS_PER_MS; // Modular arithmetic works!  :D
}
