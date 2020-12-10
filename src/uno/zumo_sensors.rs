use crate::{
    avr_async::Waiter,
    uno::timers,
    util::*,
    Uno,
};
use arduino_uno::{
    atmega328p::EXINT as ExternalInterruptRegister,
    hal::port::{
        mode::*,
        portb::*,
        portc::*,
        portd::*,
    },
    prelude::*,
};
use avr_hal_generic::avr_device;
use ufmt::uwriteln;
use void::ResultVoidExt;

const SENSOR_CHARGE_TIME_US: u16 = 10;
const SENSOR_TIMEOUT_MS: u32 = 2;

static mut SENSOR_TRIGGERED: u8 = 0; // Each bit tracks whether the corresponding sensor has registered
static mut SENSOR_VALUES: [u32; 6] = [0xffffffff; 6];

type S0 = PD5<Input<Floating>>;
type S1 = PC2<Input<Floating>>;
type S2 = PC0<Input<Floating>>;
type S3 = PB3<Input<Floating>>;
type S4 = PC3<Input<Floating>>;
type S5 = PD4<Input<Floating>>;

pub struct ZumoSensors {
    s0: Option<S0>,
    s1: Option<S1>,
    s2: Option<S2>,
    s3: Option<S3>,
    s4: Option<S4>,
    s5: Option<S5>,
    pub values: &'static [u32; 6],
}

impl ZumoSensors {
    pub fn new(s0: S0, s1: S1, s2: S2, s3: S3, s4: S4, s5: S5) -> ZumoSensors {
        unsafe {
            *PCICR = 0x00;
            *PCMSK0 = 0x08;
            *PCMSK1 = 0x0d;
            *PCMSK2 = 0x30;
        }
        ZumoSensors {
            s0: Some(s0),
            s1: Some(s1),
            s2: Some(s2),
            s3: Some(s3),
            s4: Some(s4),
            s5: Some(s5),
            values: unsafe { &SENSOR_VALUES },
        }
    }
}

impl Uno {
    pub async fn read_sensor_values(&mut self) {
        let s0 = self.sensors.s0.take().unwrap();
        let s1 = self.sensors.s1.take().unwrap();
        let s2 = self.sensors.s2.take().unwrap();
        let s3 = self.sensors.s3.take().unwrap();
        let s4 = self.sensors.s4.take().unwrap();
        let s5 = self.sensors.s5.take().unwrap();

        let mut s0 = s0.into_output(&mut self.ddr);
        let mut s1 = s1.into_output(&mut self.ddr);
        let mut s2 = s2.into_output(&mut self.ddr);
        let mut s3 = s3.into_output(&mut self.ddr);
        let mut s4 = s4.into_output(&mut self.ddr);
        let mut s5 = s5.into_output(&mut self.ddr);

        s0.set_high().void_unwrap();
        s1.set_high().void_unwrap();
        s2.set_high().void_unwrap();
        s3.set_high().void_unwrap();
        s4.set_high().void_unwrap();
        s5.set_high().void_unwrap();

        arduino_uno::delay_us(SENSOR_CHARGE_TIME_US);
        let start_time = timers::micros();
        unsafe {
            SENSOR_VALUES = [start_time; 6];
            SENSOR_TRIGGERED = 0;
        }

        toggle_pc_interrupts();
        let s0 = s0.into_floating_input(&mut self.ddr);
        let s1 = s1.into_floating_input(&mut self.ddr);
        let s2 = s2.into_floating_input(&mut self.ddr);
        let s3 = s3.into_floating_input(&mut self.ddr);
        let s4 = s4.into_floating_input(&mut self.ddr);
        let s5 = s5.into_floating_input(&mut self.ddr);

        self.sensors.s0 = Some(s0);
        self.sensors.s1 = Some(s1);
        self.sensors.s2 = Some(s2);
        self.sensors.s3 = Some(s3);
        self.sensors.s4 = Some(s4);
        self.sensors.s5 = Some(s5);

        Waiter::new(SENSOR_TIMEOUT_MS).await;
        toggle_pc_interrupts();

        // Anything that still hasn't fired at this point probably
        // isn't going to, so we just write in a dummy value.
        unsafe {
            for i in 0..SENSOR_VALUES.len() {
                if SENSOR_VALUES[i] == start_time {
                    SENSOR_VALUES[i] = 0xffffffff;
                }
            }
        }
    }
}

unsafe fn update_sensor(i: usize, is_low: bool, end_time: u32) {
    let sensor_triggered = SENSOR_TRIGGERED & (1 << i) > 0;
    if !sensor_triggered && is_low {
        SENSOR_VALUES[i] = end_time - SENSOR_VALUES[i];
        SENSOR_TRIGGERED |= 1 << i;
    }
}

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT0() {
    let s3: S3 = get_pin();
    let end_time = timers::micros_no_interrupt();
    update_sensor(3, s3.is_low().void_unwrap(), end_time);
}

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT1() {
    let (s1, s2, s4): (S1, S2, S4) = (get_pin(), get_pin(), get_pin());
    let end_time = timers::micros_no_interrupt();
    update_sensor(1, s1.is_low().void_unwrap(), end_time);
    update_sensor(2, s2.is_low().void_unwrap(), end_time);
    update_sensor(4, s4.is_low().void_unwrap(), end_time);
}

#[avr_device::interrupt(atmega328p)]
unsafe fn PCINT2() {
    let (s0, s5): (S0, S5) = (get_pin(), get_pin());
    let end_time = timers::micros_no_interrupt();
    update_sensor(0, s0.is_low().void_unwrap(), end_time);
    update_sensor(5, s5.is_low().void_unwrap(), end_time);
}
