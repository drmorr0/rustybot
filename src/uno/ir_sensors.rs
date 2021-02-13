use crate::{
    avr_async::Waiter,
    uno::timers,
    util::*,
    Uno,
};
use arduino_uno::{
    hal::port::{
        mode::*,
        portb::*,
        portc::*,
        portd::*,
    },
    pac::EXINT as ExternalInterruptRegister,
    prelude::*,
    DDR,
};
use avr_hal_generic::avr_device;
use embedded_hal::digital::v2::InputPin;
use void::ResultVoidExt;

const CALIBRATION_ITERS: u8 = 10;
const SENSOR_CHARGE_TIME_US: u16 = 10;
const SENSOR_TIMEOUT_MS: u32 = 2;
const MAX_SENSOR_READ_VALUE: u16 = 1000 * SENSOR_TIMEOUT_MS as u16;
pub const MAX_CALIBRATED_VALUE: u16 = 1000;

static mut SENSOR_TRIGGERED: u8 = 0; // Each bit tracks whether the corresponding sensor has registered
static mut SENSOR_VALUES: [u16; 6] = [u16::MAX; 6];

type S0 = PD5<Input<Floating>>;
type S1 = PC2<Input<Floating>>;
type S2 = PC0<Input<Floating>>;
type S3 = PB3<Input<Floating>>;
type S4 = PC3<Input<Floating>>;
type S5 = PD4<Input<Floating>>;

pub struct IRSensors {
    s0: Option<S0>,
    s1: Option<S1>,
    s2: Option<S2>,
    s3: Option<S3>,
    s4: Option<S4>,
    s5: Option<S5>,
    calibration_vector: [(i16, f32); 6],
    pub values: &'static [u16; 6],
}

impl IRSensors {
    pub fn new(s0: S0, s1: S1, s2: S2, s3: S3, s4: S4, s5: S5) -> IRSensors {
        unsafe {
            *PCICR = 0x00;
            *PCMSK0 = 0x08;
            *PCMSK1 = 0x0d;
            *PCMSK2 = 0x30;
        }
        IRSensors {
            s0: Some(s0),
            s1: Some(s1),
            s2: Some(s2),
            s3: Some(s3),
            s4: Some(s4),
            s5: Some(s5),
            calibration_vector: [(0, MAX_CALIBRATED_VALUE as f32); 6],
            values: unsafe { &SENSOR_VALUES },
        }
    }

    pub fn set_calibration_vector(&mut self, vector: (u16, u16, u16, u16, u16, u16, u16, u16, u16, u16, u16, u16)) {
        self.calibration_vector = [
            (
                vector.0 as i16,
                MAX_CALIBRATED_VALUE as f32 / ((vector.1 - vector.0) as f32),
            ),
            (
                vector.2 as i16,
                MAX_CALIBRATED_VALUE as f32 / ((vector.3 - vector.2) as f32),
            ),
            (
                vector.4 as i16,
                MAX_CALIBRATED_VALUE as f32 / ((vector.5 - vector.4) as f32),
            ),
            (
                vector.6 as i16,
                MAX_CALIBRATED_VALUE as f32 / ((vector.7 - vector.6) as f32),
            ),
            (
                vector.8 as i16,
                MAX_CALIBRATED_VALUE as f32 / ((vector.9 - vector.8) as f32),
            ),
            (
                vector.10 as i16,
                MAX_CALIBRATED_VALUE as f32 / ((vector.11 - vector.10) as f32),
            ),
        ];
    }

    pub async fn calibrate(&mut self, ddr: &mut DDR, dark: bool) -> [u16; 6] {
        let mut extreme_values: [u16; 6] = if dark { [0; 6] } else { [MAX_SENSOR_READ_VALUE; 6] };
        for _ in 0..CALIBRATION_ITERS {
            self.read(ddr).await;

            for j in 0..self.values.len() {
                if (dark && self.values[j] > extreme_values[j]) || (!dark && self.values[j] < extreme_values[j]) {
                    extreme_values[j] = self.values[j];
                }
            }
        }

        extreme_values
    }

    pub async fn read_calibrated(&mut self, ddr: &mut DDR) {
        self.read(ddr).await;
        unsafe {
            for i in 0..SENSOR_VALUES.len() {
                let v = (SENSOR_VALUES[i] as i16 - self.calibration_vector[i].0) as f32 * self.calibration_vector[i].1;
                SENSOR_VALUES[i] = match v {
                    v if v < 0.0 => 0,
                    v if v > MAX_CALIBRATED_VALUE as f32 => MAX_CALIBRATED_VALUE,
                    v => v as u16,
                }
            }
        }
    }

    pub async fn read(&mut self, ddr: &mut DDR) {
        let s0 = self.s0.take().unwrap();
        let s1 = self.s1.take().unwrap();
        let s2 = self.s2.take().unwrap();
        let s3 = self.s3.take().unwrap();
        let s4 = self.s4.take().unwrap();
        let s5 = self.s5.take().unwrap();

        let mut s0 = s0.into_output(ddr);
        let mut s1 = s1.into_output(ddr);
        let mut s2 = s2.into_output(ddr);
        let mut s3 = s3.into_output(ddr);
        let mut s4 = s4.into_output(ddr);
        let mut s5 = s5.into_output(ddr);

        s0.set_high().void_unwrap();
        s1.set_high().void_unwrap();
        s2.set_high().void_unwrap();
        s3.set_high().void_unwrap();
        s4.set_high().void_unwrap();
        s5.set_high().void_unwrap();

        arduino_uno::delay_us(SENSOR_CHARGE_TIME_US);
        let start_time = timers::micros() as u16; // modular arithemtic makes this work even when it rolls over
        unsafe {
            SENSOR_VALUES = [start_time; 6];
            SENSOR_TRIGGERED = 0;
        }

        toggle_pc_interrupts();
        let s0 = s0.into_floating_input(ddr);
        let s1 = s1.into_floating_input(ddr);
        let s2 = s2.into_floating_input(ddr);
        let s3 = s3.into_floating_input(ddr);
        let s4 = s4.into_floating_input(ddr);
        let s5 = s5.into_floating_input(ddr);

        self.s0 = Some(s0);
        self.s1 = Some(s1);
        self.s2 = Some(s2);
        self.s3 = Some(s3);
        self.s4 = Some(s4);
        self.s5 = Some(s5);

        Waiter::new(SENSOR_TIMEOUT_MS).await;
        toggle_pc_interrupts();

        // Anything that still hasn't fired at this point probably
        // isn't going to, so we just write in a dummy value.
        unsafe {
            for i in 0..SENSOR_VALUES.len() {
                if SENSOR_VALUES[i] == start_time || SENSOR_VALUES[i] > MAX_SENSOR_READ_VALUE {
                    SENSOR_VALUES[i] = MAX_SENSOR_READ_VALUE;
                }
            }
        }
    }
}

unsafe fn update_sensor(i: usize, is_low: bool, end_time: u16) {
    let sensor_triggered = SENSOR_TRIGGERED & (1 << i) > 0;
    if !sensor_triggered && is_low {
        SENSOR_VALUES[i] = end_time - SENSOR_VALUES[i];
        SENSOR_TRIGGERED |= 1 << i;
    }
}

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT0() {
    let s3: S3 = get_pin();
    let end_time = timers::micros_no_interrupt() as u16;
    update_sensor(3, s3.is_low().void_unwrap(), end_time);
}

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT1() {
    let (s1, s2, s4): (S1, S2, S4) = (get_pin(), get_pin(), get_pin());
    let end_time = timers::micros_no_interrupt() as u16;
    update_sensor(1, s1.is_low().void_unwrap(), end_time);
    update_sensor(2, s2.is_low().void_unwrap(), end_time);
    update_sensor(4, s4.is_low().void_unwrap(), end_time);
}

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT2() {
    let (s0, s5): (S0, S5) = (get_pin(), get_pin());
    let end_time = timers::micros_no_interrupt() as u16;
    update_sensor(0, s0.is_low().void_unwrap(), end_time);
    update_sensor(5, s5.is_low().void_unwrap(), end_time);
}
