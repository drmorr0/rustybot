#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]

mod motor;
mod timers;
mod zumo_sensors;

use crate::{
    motor::{
        MotorController,
        UPDATE_DELAY_US,
    },
    zumo_sensors::ZumoSensors,
};
use arduino_uno::{
    atmega328p::TC0 as Timer0,
    hal::{
        clock::MHz16,
        port::{
            mode::*,
            portb::*,
            portd::*,
        },
        pwm,
        usart::Usart0,
    },
    prelude::*,
};
use avr_hal_generic::avr_device;
use ufmt::uwriteln;

struct Uno {
    serial: Usart0<MHz16, Floating>,
    timer0: Timer0,

    ddr: arduino_uno::DDR,
    led: PB5<Output>,
    left_motor: MotorController<PB0<Output>, PB2<Pwm<pwm::Timer1Pwm>>>,
    right_motor: MotorController<PD7<Output>, PB1<Pwm<pwm::Timer1Pwm>>>,
    zumo_sensors: ZumoSensors,
}

impl Uno {
    pub fn init() -> Uno {
        let board = arduino_uno::Peripherals::take().unwrap();
        let pins = arduino_uno::Pins::new(board.PORTB, board.PORTC, board.PORTD);
        let serial = arduino_uno::Serial::new(board.USART0, pins.d0, pins.d1.into_output(&pins.ddr), 57600);
        let led = pins.d13.into_output(&pins.ddr);
        unsafe {
            avr_device::interrupt::enable();
        }

        let mut pwm_timer = pwm::Timer1Pwm::new(board.TC1, pwm::Prescaler::Prescale64);
        let left_motor = MotorController::new(
            pins.d8.into_output(&pins.ddr),
            pins.d10.into_output(&pins.ddr).into_pwm(&mut pwm_timer),
        );
        let right_motor = MotorController::new(
            pins.d7.into_output(&pins.ddr),
            pins.d9.into_output(&pins.ddr).into_pwm(&mut pwm_timer),
        );
        Uno::init_timers(&board.TC0);
        Uno {
            serial,
            timer0: board.TC0,

            ddr: pins.ddr,
            led,
            left_motor,
            right_motor,
            zumo_sensors: ZumoSensors {
                s0: Some(pins.d5),
                s1: Some(pins.a2),
                s2: Some(pins.a0),
                s3: Some(pins.d11),
                s4: Some(pins.a3),
                s5: Some(pins.d4),
            },
        }
    }
}

enum State {
    Forward,
    Backward,
    Stopped,
}

#[arduino_uno::entry]
fn main() -> ! {
    let mut uno = Uno::init();

    let mut state_change_time = unsafe { uno.micros() };
    let mut last_update_time = state_change_time;
    let mut state = State::Forward;
    uno.left_motor.set(1.0);
    uno.right_motor.set(1.0);
    loop {
        let now = unsafe { uno.micros() };
        if now >= last_update_time + UPDATE_DELAY_US {
            uno.left_motor.update();
            uno.right_motor.update();
            last_update_time = now;
        }
        if now >= state_change_time + 1000000 {
            uno.led.toggle();
            state = match state {
                State::Forward => {
                    uno.left_motor.set(-1.0);
                    uno.right_motor.set(-1.0);
                    state_change_time = now;
                    State::Backward
                },
                State::Backward => {
                    uno.left_motor.set(0.0);
                    uno.right_motor.set(0.0);
                    State::Stopped
                },
                _ => State::Stopped,
            };
        }
    }

    let sensor_values = uno.read_sensors();
    uwriteln!(
        &mut uno.serial,
        "{} {} {} {} {} {}",
        sensor_values[0],
        sensor_values[1],
        sensor_values[2],
        sensor_values[3],
        sensor_values[4],
        sensor_values[5]
    )
    .void_unwrap();

    loop {}
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
