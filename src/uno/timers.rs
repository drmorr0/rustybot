use crate::avr_async::TimedWaker;
use arduino_uno::atmega328p::TC0 as Timer0;
use avr_device::interrupt::free as critical_section;
use avr_hal_generic::avr_device;
use core::task::Waker;
use heapless::{
    consts::U8,
    Vec,
};

static mut TIMER0_OVF_COUNT: u32 = 0;
static mut ELAPSED_MS: u32 = 0;

// Using prescale_64 gives 64 / 16000000 = 4us per tick;
// The timer overflows every 4 * 256 = 1024us
const TIMER0_TICK_US: u32 = 4;
const TIMER0_OVF_US: u32 = 1024;
const TIMER0_TICKS_PER_MS: u8 = 250; // 1000us / (4us per tick) = 250 ticks/ms

const TCNT0: *const u8 = 0x46 as *const u8;
const OCR0A: *mut u8 = 0x47 as *mut u8;

static mut NEXT_WAKEUP_TIME: u32 = 0xffffff;
static mut WAITERS_LEN: usize = 0;
static mut WAITERS: [Option<TimedWaker>; 8] = [None; 8];

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
    critical_section(|_| {
        // I don't want to have to pass around the uno object to get access to Timer0 or whatever
        // so we just use the raw address.  See note below about "in/out" vs "ld/st" instructions.
        unsafe { (*TCNT0 as u32) * TIMER0_TICK_US + TIMER0_OVF_COUNT * TIMER0_OVF_US }
    })
}

// This will overflow after about 49 days
pub fn millis() -> u32 {
    critical_section(|_| unsafe { ELAPSED_MS })
}

pub fn register_timed_waker(trigger_time_ms: u32, waker: Waker) {
    critical_section(|_| unsafe {
        WAITERS[WAITERS_LEN] = Some(TimedWaker { trigger_time_ms, waker });
        WAITERS_LEN += 1;
        if trigger_time_ms < NEXT_WAKEUP_TIME {
            NEXT_WAKEUP_TIME = trigger_time_ms;
        }
    });
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_OVF() {
    TIMER0_OVF_COUNT += 1;
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_COMPA() {
    ELAPSED_MS += 1;

    // If we have any awaitable tasks that are ready to wake up, send a wake-up signal
    if ELAPSED_MS >= NEXT_WAKEUP_TIME {
        NEXT_WAKEUP_TIME = 0xffffffff;
        for i in 0..8 {
            if let Some(waiter) = WAITERS[i].take() {
                if ELAPSED_MS >= waiter.trigger_time_ms {
                    waiter.waker.wake();
                    if let Some(swap_waiter) = WAITERS[WAITERS_LEN - 1].take() {
                        WAITERS[i] = Some(swap_waiter);
                    }
                    WAITERS_LEN -= 1;
                } else if waiter.trigger_time_ms < NEXT_WAKEUP_TIME {
                    NEXT_WAKEUP_TIME = waiter.trigger_time_ms;
                    WAITERS[i] = Some(waiter);
                }
            }
        }
    };

    // We don't have access to the Timer0 object here, so just use its raw memory location.
    // The AVR spec specifies that with the "in/out" instructions, you must subtract 0x20 from
    // the address; inspecting the compiler output shows that it uses "in/out" instructions _and_
    // it automagically subtracts 0x20, so we use here the 0x47 address for ld/st.
    *OCR0A += TIMER0_TICKS_PER_MS; // Modular arithmetic works!  :D
}
