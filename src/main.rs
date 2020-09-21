#![no_std]
#![no_main]
#![feature(llvm_asm)]
#![feature(abi_avr_interrupt)]

// Pull in the panic handler from panic-halt
extern crate panic_halt;
mod motor;
mod timers;
mod zumo_sensors;

use crate::{
    motor::{
        LeftMotorController,
        RightMotorController,
    },
    zumo_sensors::ZumoSensors,
};
use arduino_uno::{
    atmega328p::TC0 as Timer0,
    hal::{
        clock::MHz16,
        port::mode::*,
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
    left_motor: LeftMotorController,
    right_motor: RightMotorController,
    zumo_sensors: ZumoSensors,
}

impl Uno {
    pub fn init() -> Uno {
        let board = arduino_uno::Peripherals::take().unwrap();
        let pins = arduino_uno::Pins::new(board.PORTB, board.PORTC, board.PORTD);
        let serial = arduino_uno::Serial::new(board.USART0, pins.d0, pins.d1.into_output(&pins.ddr), 57600);
        unsafe {
            avr_device::interrupt::enable();
        }

        let mut pwm_timer = pwm::Timer1Pwm::new(board.TC1, pwm::Prescaler::Prescale64);
        let left_motor = LeftMotorController::new(pins.d8, pins.d10, &pins.ddr, &mut pwm_timer);
        let right_motor = RightMotorController::new(pins.d7, pins.d9, &pins.ddr, &mut pwm_timer);
        Uno::init_timers(&board.TC0);
        Uno {
            serial,
            timer0: board.TC0,

            ddr: pins.ddr,
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

#[arduino_uno::entry]
fn main() -> ! {
    let mut uno = Uno::init();

    uno.left_motor.set(1.0);
    uno.right_motor.set(1.0);
    arduino_uno::delay_ms(1000);

    uno.left_motor.set(-1.0);
    uno.right_motor.set(-1.0);
    arduino_uno::delay_ms(1000);

    uno.left_motor.set(0.0);
    uno.right_motor.set(0.0);
    loop {
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
        arduino_uno::delay_ms(1000);
    }
}

/*#[arduino_uno::entry]
fn main() -> ! {
    let mut dp = arduino_uno::Peripherals::take().unwrap();
    let mut pins = arduino_uno::Pins::new(dp.PORTB, dp.PORTC, dp.PORTD);

    let mut right_motor_direction = pins.d7.into_output(&mut pins.ddr);
    let mut left_motor_direction = pins.d8.into_output(&mut pins.ddr);

    let mut timer1 = pwm::Timer1Pwm::new(dp.TC1, pwm::Prescaler::Prescale64);
    let mut right_motor_speed = pins.d9.into_output(&mut pins.ddr).into_pwm(&mut timer1);
    let mut left_motor_speed = pins.d10.into_output(&mut pins.ddr).into_pwm(&mut timer1);

    right_motor_direction.set_low().void_unwrap();
    left_motor_direction.set_low().void_unwrap();
    right_motor_speed.enable();
    left_motor_speed.enable();

    loop {
        for i in 0..10 {
            right_motor_speed.set_duty(i * 20);
            left_motor_speed.set_duty(i * 20);
            arduino_uno::delay_ms(250);
        }
        for i in (0..10).rev() {
            right_motor_speed.set_duty(i * 20);
            left_motor_speed.set_duty(i * 20);
            arduino_uno::delay_ms(250);
        }
    }
}*/
