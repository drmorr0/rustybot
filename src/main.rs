#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]
#![feature(never_type)]
#![feature(async_closure)]
#![feature(panic_info_message)]
#![feature(fmt_as_str)]

mod avr_async;
mod mem;
mod state_machine;
mod uno;

use crate::{
    avr_async::Executor,
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

use crate::{
    avr_async::waiter::*,
    uno::timers::*,
};

#[arduino_uno::entry]
fn main() -> ! {
    let mut uno = Uno::init();
    uwriteln!(uno.serial, "re-entered main");
    let executor = Executor::get();

    executor.add_driver(uno.led_driver);
    executor.run(&mut uno.serial);

    loop {}
    // let mut current_state: State = ExplorationState::new();
    // loop {
    //     let now = unsafe { uno.micros() };
    //     if let Some(s) = current_state.update(&mut uno, now) {
    //         current_state = s;
    //     }
    //     uno.update();
    //     arduino_uno::delay_ms(10);
    // }
}

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let mut serial: arduino_uno::Serial<Floating> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };
    let mut led: PB5<Output> = unsafe { core::mem::MaybeUninit::uninit().assume_init() };

    uwriteln!(&mut serial, "Firmware panic!\r").void_unwrap();

    if let Some(loc) = info.location() {
        ufmt::uwriteln!(&mut serial, "  At {}:{}:{}\r", loc.file(), loc.line(), loc.column(),).void_unwrap();
    }
    if let Some(message_args) = info.message() {
        if let Some(message) = message_args.as_str() {
            ufmt::uwriteln!(&mut serial, "    {}\r", message);
        }
    }
    let TIMSK0 = 0x6E as *const u8;
    unsafe {
        uwriteln!(
            &mut serial,
            "{} {} {} {} {}",
            TIMER0_CMPB_ITERS,
            TIMER0_CMPB_NEEDED_ITERS,
            TIMER0_CMPB_REMAINDER,
            *TIMSK0,
            *OCR0B
        )
        .unwrap();
    }
    loop {
        led.set_high().void_unwrap();
        arduino_uno::delay_ms(100);
        led.set_low().void_unwrap();
        arduino_uno::delay_ms(100);
    }
}
