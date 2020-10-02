#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]

mod state_machine;
mod uno;

use crate::{
    state_machine::{
        ExplorationState,
        State,
        StateObject,
    },
    uno::Uno,
};
use arduino_uno::{
    hal::port::{
        mode::*,
        portb::*,
    },
    prelude::*,
};
use ufmt::uwriteln;

#[arduino_uno::entry]
fn main() -> ! {
    let mut uno = Uno::init();

    let mut current_state: State = ExplorationState::new();
    loop {
        let now = unsafe { uno.micros() };
        if let Some(s) = current_state.update(&mut uno, now) {
            current_state = s;
        }
        uno.update();
        arduino_uno::delay_ms(10);
    }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut serial: arduino_uno::Serial<Floating> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    let mut led: PB5<Output> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };

    uwriteln!(&mut serial, "Firmware panic!\r").void_unwrap();

    if let Some(loc) = info.location() {
        ufmt::uwriteln!(&mut serial, "  At {}:{}:{}\r", loc.file(), loc.line(), loc.column(),).void_unwrap();
    }
    loop {
        led.set_high().void_unwrap();
        arduino_uno::delay_ms(100);
        led.set_low().void_unwrap();
        arduino_uno::delay_ms(100);
    }
}
