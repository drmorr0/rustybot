use crate::Uno;
use arduino_uno::{
    hal::port::{
        mode::*,
        portb::*,
        portc::*,
        portd::*,
    },
    prelude::*,
};

const SENSOR_TIMEOUT: u16 = 2000;

pub struct ZumoSensors {
    pub s0: Option<PD5<Input<Floating>>>,
    pub s1: Option<PC2<Input<Floating>>>,
    pub s2: Option<PC0<Input<Floating>>>,
    pub s3: Option<PB3<Input<Floating>>>,
    pub s4: Option<PC3<Input<Floating>>>,
    pub s5: Option<PD4<Input<Floating>>>,
}

impl Uno {
    pub fn read_sensors(&mut self) -> [u16; 6] {
        let s0 = self.zumo_sensors.s0.take().unwrap();
        let s1 = self.zumo_sensors.s1.take().unwrap();
        let s2 = self.zumo_sensors.s2.take().unwrap();
        let s3 = self.zumo_sensors.s3.take().unwrap();
        let s4 = self.zumo_sensors.s4.take().unwrap();
        let s5 = self.zumo_sensors.s5.take().unwrap();

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

        arduino_uno::delay_ms(10);

        let s0 = s0.into_floating_input(&mut self.ddr);
        let s1 = s1.into_floating_input(&mut self.ddr);
        let s2 = s2.into_floating_input(&mut self.ddr);
        let s3 = s3.into_floating_input(&mut self.ddr);
        let s4 = s4.into_floating_input(&mut self.ddr);
        let s5 = s5.into_floating_input(&mut self.ddr);

        let mut result = [SENSOR_TIMEOUT; 6];

        let start_time = unsafe { self.micros() };
        loop {
            let time = (unsafe { self.micros() } - start_time) as u16;
            if time >= SENSOR_TIMEOUT {
                break;
            }

            if s0.is_low().void_unwrap() && time < result[0] {
                result[0] = time;
            }
            if s1.is_low().void_unwrap() && time < result[1] {
                result[1] = time;
            }
            if s2.is_low().void_unwrap() && time < result[2] {
                result[2] = time;
            }
            if s3.is_low().void_unwrap() && time < result[3] {
                result[3] = time;
            }
            if s4.is_low().void_unwrap() && time < result[4] {
                result[4] = time;
            }
            if s5.is_low().void_unwrap() && time < result[5] {
                result[5] = time;
            }
        }

        self.zumo_sensors.s0 = Some(s0);
        self.zumo_sensors.s1 = Some(s1);
        self.zumo_sensors.s2 = Some(s2);
        self.zumo_sensors.s3 = Some(s3);
        self.zumo_sensors.s4 = Some(s4);
        self.zumo_sensors.s5 = Some(s5);

        result
    }
}
