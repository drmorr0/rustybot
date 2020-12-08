mod motor;
pub mod timers;
mod zumo_sensors;

use crate::{
    avr_async::{
        Executor,
        Waiter,
    },
    mem::Allocator,
    uno::{
        motor::get_motor_driver,
        zumo_sensors::ZumoSensors,
    },
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
use core::{
    cell::RefCell,
    future::Future,
};
use micromath::F32Ext;
use ufmt::{
    uwrite,
    uwriteln,
};
use void::ResultVoidExt;

pub use motor::MotorController;

pub struct Uno {
    pub serial: Usart0<MHz16, Floating>,
    timer0: Timer0,

    ddr: arduino_uno::DDR,
    pub motor_controller: &'static RefCell<MotorController>,
    pub sensors: ZumoSensors,
}

fn led_driver(mut led: PB5<Output>) -> &'static mut dyn Future<Output = !> {
    let future = async move || loop {
        led.toggle().void_unwrap();
        Waiter::new(750).await;
    };
    Allocator::get().new(future())
}

impl Uno {
    pub fn init(executor: &mut Executor) -> &'static mut Uno {
        let board = arduino_uno::Peripherals::take().unwrap();
        let pins = arduino_uno::Pins::new(board.PORTB, board.PORTC, board.PORTD);
        let serial = arduino_uno::Serial::new(board.USART0, pins.d0, pins.d1.into_output(&pins.ddr), 57600);
        let led = pins.d13.into_output(&pins.ddr);
        unsafe {
            avr_device::interrupt::enable();
            *(0x53 as *mut u8) = 0x01; // Turn on "idle sleep mode"
        }

        let mut pwm_timer = pwm::Timer1Pwm::new(board.TC1, pwm::Prescaler::Prescale64);
        let motor_controller = MotorController::new();
        timers::init_timers(&board.TC0);
        executor.add_async_driver(led_driver(led));
        executor.add_async_driver(get_motor_driver(
            motor_controller,
            pins.d8.into_output(&pins.ddr),
            pins.d10.into_output(&pins.ddr).into_pwm(&mut pwm_timer),
            pins.d7.into_output(&pins.ddr),
            pins.d9.into_output(&pins.ddr).into_pwm(&mut pwm_timer),
        ));
        Allocator::get().new(Uno {
            serial,
            timer0: board.TC0,

            ddr: pins.ddr,
            motor_controller,
            sensors: ZumoSensors::new(pins.d5, pins.a2, pins.a0, pins.d11, pins.a3, pins.d4),
        })
    }

    pub fn write_state(&mut self) {
        uwriteln!(
            &mut self.serial,
            "sensors = [{}, {}, {}, {}, {}, {}]",
            self.sensors.values[0],
            self.sensors.values[1],
            self.sensors.values[2],
            self.sensors.values[3],
            self.sensors.values[4],
            self.sensors.values[5],
        )
        .void_unwrap();
        // let now = timers::millis();
        // let upper_padding = 5 - ((((now >> 16) as u16) as f32).log10() as u16);
        // let lower_padding = 5 - (((now as u16) as f32).log10() as u16);
        // for _ in 0..upper_padding {
        //     nb::block!(self.serial.write('0' as u8)).void_unwrap();
        // }
        // uwrite!(&mut self.serial, "{}", (now >> 16) as u16).void_unwrap();
        // for _ in 0..lower_padding {
        //     nb::block!(self.serial.write('0' as u8)).void_unwrap();
        // }
        // uwrite!(&mut self.serial, "{}", now as u16).void_unwrap();
        // if let Ok(mc) = self.motor_controller.try_borrow() {
        //     uwriteln!(
        //         &mut self.serial,
        //         "{} {}",
        //         (mc.left_target * 255.0) as i16,
        //         (mc.right_target * 255.0) as i16,
        //     )
        //     .void_unwrap();
        // } else {
        //     uwriteln!(&mut self.serial, "unavailable").void_unwrap();
        // }
    }
}
