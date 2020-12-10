use core::mem::MaybeUninit;
use embedded_hal::digital::v2::InputPin;

// These constants are useful for when we don't have access to the "uno" object but we still
// need to do register accesses.  You may need to wrap these in read/write_volatile commands to
// prevent various compiler optimizations.
//
// The AVR spec specifies that with the "in/out" instructions, you must subtract 0x20 from
// the address; inspecting the compiler output shows that it uses "in/out" instructions _and_
// it automagically subtracts 0x20, so we use here the 0x47 address for ld/st.
pub const PCICR: *mut u8 = 0x68 as *mut u8;
pub const PCMSK0: *mut u8 = 0x6b as *mut u8;
pub const PCMSK1: *mut u8 = 0x6c as *mut u8;
pub const PCMSK2: *mut u8 = 0x6d as *mut u8;
pub const TCNT0: *const u8 = 0x46 as *const u8;
pub const TIFR0: *const u8 = 0x35 as *const u8;
pub const OCR0A: *mut u8 = 0x47 as *mut u8;

pub fn get_pin<T: InputPin>() -> T {
    unsafe { MaybeUninit::uninit().assume_init() }
}

pub fn toggle_pc_interrupts() {
    unsafe {
        *PCICR ^= 0x07;
    }
}
