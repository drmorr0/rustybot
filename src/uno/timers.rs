use crate::Uno;
use arduino_uno::atmega328p::TC0 as Timer0;
use avr_device::interrupt::free as critical_section;
use avr_hal_generic::avr_device;

static mut TIMER0_OVF_COUNT: u32 = 0;
static mut ELAPSED_MS: u32 = 0;

// Using prescale_64 gives 64 / 16000000 = 4us per tick;
// The timer overflows every 4 * 256 = 1024us
const TIMER0_TICK_US: u32 = 4;
const TIMER0_OVF_US: u32 = TIMER0_TICK_US * 256;
const TIMER0_TICKS_PER_MS: u8 = 250; // 1000us / (4us per tick) = 250 ticks/ms

pub fn init_timers(t0: &Timer0) {
    t0.tccr0b.write(|w| w.cs0().prescale_64());
    t0.tcnt0.write(|w| unsafe { w.bits(0) });
    t0.timsk0.write(|w| unsafe { w.bits(0x03) }); // enable overflow interrupt and COMPA interrupt
    t0.ocr0a.write(|w| unsafe { w.bits(TIMER0_TICKS_PER_MS) });
}

impl Uno {
    // In *theory* this wouldn't overflow for (255 * 1024 + 4294967295 * 1024)us, but since
    // it can only return a 32-bit integer, it actually wraps around after about 71 minutes.
    //
    // Bummer.
    pub unsafe fn micros(&self) -> u32 {
        critical_section(|_| {
            (self.timer0.tcnt0.read().bits() as u32) * TIMER0_TICK_US + TIMER0_OVF_COUNT * TIMER0_OVF_US
        })
    }

    // This will overflow after about 49 days
    pub unsafe fn millis(&self) -> u32 {
        // There's no particular reason this _has_ to be a method except for consistency
        critical_section(|_| ELAPSED_MS)
    }
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_OVF() {
    TIMER0_OVF_COUNT += 1;
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_COMPA() {
    ELAPSED_MS += 1;

    // We don't have access to the Timer0 object here, so just use its raw memory location.
    // The AVR spec specifies that with the "in/out" instructions, you must subtract 0x20 from
    // the address; inspecting the compiler output shows that it uses "in/out" instructions _and_
    // it automagically subtracts 0x20, so we use here the 0x47 address for ld/st.
    let ocr0a = 0x47 as *mut u8;
    *ocr0a += TIMER0_TICKS_PER_MS; // Modular arithmetic works!  :D
}
