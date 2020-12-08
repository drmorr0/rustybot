#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]
#![feature(never_type)] // Used for futures that never return
#![feature(async_closure)] // drivers are implemented as async closures
#![feature(panic_info_message)] // Get a usable message when we panic (sometimes)
#![feature(fmt_as_str)] // Convert panic message args to a string
#![feature(maybe_uninit_ref)] // Get a mutable reference to a maybe-uninit driver
#![feature(const_ptr_offset)] // Get a pointer to the MEMORY (fake heap)
#![feature(const_maybe_uninit_as_ptr)]
#![allow(dead_code)]
#![allow(unused_imports)]

mod avr_async;
mod mem;
mod state_machine;
mod uno;

use crate::{
    avr_async::{
        Executor,
        Waiter,
    },
    mem::Allocator,
    state_machine::build_state_machine,
    uno::{
        MotorController,
        Uno,
    },
};
use arduino_uno::{
    hal::port::{
        mode::*,
        portb::*,
    },
    prelude::*,
};
use core::cell::RefCell;
use ufmt::uwriteln;

#[arduino_uno::entry]
fn main() -> ! {
    let mut executor = Executor::get();
    let uno = Uno::init(&mut executor);
    uno.motor_controller.borrow_mut().left_target = 0.0;
    uno.motor_controller.borrow_mut().right_target = 0.0;
    executor.add_async_driver(build_state_machine(uno));

    executor.run();
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
            ufmt::uwriteln!(&mut serial, "    {}\r", message).void_unwrap();
        }
    }

    loop {
        led.set_high().void_unwrap();
        arduino_uno::delay_ms(100);
        led.set_low().void_unwrap();
        arduino_uno::delay_ms(100);
    }
}
