use crate::Uno;
use arduino_uno::atmega328p::TC0;
use avr_hal_generic::avr_device;

static mut TIMER0_OVF_COUNT: u32 = 0;
static mut MILLIS_COUNTER: u16 = 0;
const TIMER0_TICK_MICROS: u32 = 4;
const TIMER0_OVF_MICROS: u32 = TIMER0_TICK_MICROS * 255;

impl Uno {
    pub fn init_timers(t0: &TC0) {
        t0.tccr0b.write(|w| w.cs0().prescale_64());
        t0.tcnt0.write(|w| unsafe { w.bits(0) });
        t0.timsk0.write(|w| unsafe { w.bits(1) });
    }

    pub unsafe fn micros(&self) -> u32 {
        (self.timer0.tcnt0.read().bits() as u32) * TIMER0_TICK_MICROS + TIMER0_OVF_COUNT * TIMER0_OVF_MICROS
    }

    pub fn millis() -> u8 {
        unimplemented!();
    }
}

#[avr_device::interrupt(atmega328p)]
unsafe fn TIMER0_OVF() {
    TIMER0_OVF_COUNT += 1;
}
