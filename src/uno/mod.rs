pub mod eeprom;
mod imu;
mod ir_sensors;
pub mod motor;
mod pushbutton;
pub mod timers;

use crate::{
    avr_async::{
        Executor,
        Waiter,
    },
    mem::Allocator,
    uno::{
        eeprom::{
            IMU_X_MAX_ADDR,
            IMU_X_MIN_ADDR,
            IMU_Y_MAX_ADDR,
            IMU_Y_MIN_ADDR,
        },
        imu::IMU,
        ir_sensors::IRSensors,
        pushbutton::Pushbutton,
    },
};
use arduino_uno::{
    hal::{
        clock::MHz16,
        port::{
            mode::*,
            portb::*,
            portd::*,
        },
        pwm,
        usart::{
            Baudrate,
            Usart0,
        },
    },
    pac::{
        EEPROM,
        TC0 as Timer0,
    },
    prelude::*,
};
use avr_hal_generic::avr_device;
use core::{
    cell::RefCell,
    future::Future,
};
use micromath::F32Ext;
use void::ResultVoidExt;

pub use motor::MotorController;

const SERIAL_BAUD: u32 = 57600;
const I2C_SPEED: u32 = 25000;

pub struct Uno {
    pub serial: Usart0<MHz16, Floating>,
    timer0: Timer0,

    ddr: arduino_uno::DDR,
    eeprom: EEPROM,
    pub imu: IMU,
    pub ir_sensors: IRSensors,
    pub motor_controller: &'static MotorController,
    pub pushbutton: Pushbutton,
    pub led: PB5<Output>,
}

impl Uno {
    pub fn init(executor: &mut Executor) -> &'static mut Uno {
        let board = arduino_uno::Peripherals::take().unwrap();
        let pins = arduino_uno::Pins::new(board.PORTB, board.PORTC, board.PORTD);
        let serial = arduino_uno::Serial::new(
            board.USART0,
            pins.d0,
            pins.d1.into_output(&pins.ddr),
            SERIAL_BAUD.into_baudrate(),
        );
        let i2c = arduino_uno::I2cMaster::new(
            board.TWI,
            pins.a4.into_pull_up_input(&pins.ddr),
            pins.a5.into_pull_up_input(&pins.ddr),
            I2C_SPEED,
        );

        let led = pins.d13.into_output(&pins.ddr);
        let pushbutton = Pushbutton::new(pins.d12.into_pull_up_input(&pins.ddr));
        unsafe {
            avr_device::interrupt::enable();
            *(0x53 as *mut u8) = 0x01; // Turn on "idle sleep mode"
        }

        let mut pwm_timer = pwm::Timer1Pwm::new(board.TC1, pwm::Prescaler::Prescale64);
        let motor_controller = MotorController::new(
            pins.d8.into_output(&pins.ddr),
            pins.d10.into_output(&pins.ddr).into_pwm(&mut pwm_timer),
            pins.d7.into_output(&pins.ddr),
            pins.d9.into_output(&pins.ddr).into_pwm(&mut pwm_timer),
        );
        timers::init_timers(&board.TC0);
        executor.add_async_driver(motor_controller.get_motor_driver());
        Allocator::get().new(Uno {
            serial,
            timer0: board.TC0,

            ddr: pins.ddr,
            eeprom: board.EEPROM,
            imu: IMU::new(i2c),
            ir_sensors: IRSensors::new(pins.d5, pins.a2, pins.a0, pins.d11, pins.a3, pins.d4),
            motor_controller,
            pushbutton,
            led,
        })
    }

    pub async fn load_calibration_data(&mut self) {
        let (imu_x_min, imu_x_max, imu_y_min, imu_y_max) = (
            self.read_eeprom_u16(IMU_X_MIN_ADDR).await as i16,
            self.read_eeprom_u16(IMU_X_MAX_ADDR).await as i16,
            self.read_eeprom_u16(IMU_Y_MIN_ADDR).await as i16,
            self.read_eeprom_u16(IMU_Y_MAX_ADDR).await as i16,
        );
        self.imu
            .set_calibration_vector(imu_x_min, imu_x_max, imu_y_min, imu_y_max);
    }
}
