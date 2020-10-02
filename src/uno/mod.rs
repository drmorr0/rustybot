mod motor;
mod timers;
mod zumo_sensors;

use crate::uno::{
    motor::MotorController,
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
};
use avr_hal_generic::avr_device;
use embedded_hal::prelude::*;
use micromath::F32Ext;
use ufmt::{
    uwrite,
    uwriteln,
};
use void::ResultVoidExt;

pub struct Uno {
    pub serial: Usart0<MHz16, Floating>,
    timer0: Timer0,

    ddr: arduino_uno::DDR,
    pub led: PB5<Output>,
    pub left_motor: MotorController<PB0<Output>, PB2<Pwm<pwm::Timer1Pwm>>>,
    pub right_motor: MotorController<PD7<Output>, PB1<Pwm<pwm::Timer1Pwm>>>,
    pub sensors: ZumoSensors,
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
        timers::init_timers(&board.TC0);
        Uno {
            serial,
            timer0: board.TC0,

            ddr: pins.ddr,
            led,
            left_motor,
            right_motor,
            sensors: ZumoSensors::new(pins.d5, pins.a2, pins.a0, pins.d11, pins.a3, pins.d4),
        }
    }

    pub fn update(&mut self) {
        self.read_sensors();
        self.left_motor.update(unsafe { self.micros() });
        self.right_motor.update(unsafe { self.micros() });
        self.write_state(unsafe { self.micros() });
    }

    pub fn write_state(&mut self, now: u32) {
        let upper_padding = 5 - ((((now >> 16) as u16) as f32).log10() as u16);
        let lower_padding = 5 - (((now as u16) as f32).log10() as u16);
        for _ in 0..upper_padding {
            nb::block!(self.serial.write('0' as u8)).void_unwrap();
        }
        uwrite!(&mut self.serial, "{}", (now >> 16) as u16).void_unwrap();
        for _ in 0..lower_padding {
            nb::block!(self.serial.write('0' as u8)).void_unwrap();
        }
        uwrite!(&mut self.serial, "{}", now as u16).void_unwrap();
        uwriteln!(
            &mut self.serial,
            ": sensors = [{} {} {} {} {} {}]; motors = {}/{} {}/{}",
            self.sensors.values[0],
            self.sensors.values[1],
            self.sensors.values[2],
            self.sensors.values[3],
            self.sensors.values[4],
            self.sensors.values[5],
            (self.left_motor.current_value * 255.0) as i16,
            (self.left_motor.target_value * 255.0) as i16,
            (self.right_motor.current_value * 255.0) as i16,
            (self.right_motor.target_value * 255.0) as i16,
        )
        .void_unwrap();
    }
}
