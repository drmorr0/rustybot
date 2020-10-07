use crate::avr_async::{
    Driver,
    Waiter,
};
use arduino_uno::{
    hal::port::{
        mode::Output,
        portb::PB5,
    },
    prelude::*,
};

pub static mut TOGGLE_COUNT: u8 = 0;

pub fn make_led_driver(mut led: PB5<Output>) -> Driver {
    let future = async move || loop {
        led.toggle();
        Waiter::new(1000).await;
    };
    Driver::new(future())
}
