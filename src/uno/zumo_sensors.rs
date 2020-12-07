use crate::{
    uno::timers,
    yielder,
    Uno,
};
use arduino_uno::{
    hal::port::{
        mode::*,
        portb::*,
        portc::*,
        portd::*,
    },
    prelude::*,
};
use ufmt::uwriteln;
use void::ResultVoidExt;

const SENSOR_CHARGE_TIME_US: u16 = 10;
const SENSOR_TIMEOUT_US: u32 = 500;

pub struct ZumoSensors {
    pub s0: Option<PD5<Input<Floating>>>,
    pub s1: Option<PC2<Input<Floating>>>,
    pub s2: Option<PC0<Input<Floating>>>,
    pub s3: Option<PB3<Input<Floating>>>,
    pub s4: Option<PC3<Input<Floating>>>,
    pub s5: Option<PD4<Input<Floating>>>,
}

impl ZumoSensors {
    pub fn new(
        s0: PD5<Input<Floating>>,
        s1: PC2<Input<Floating>>,
        s2: PC0<Input<Floating>>,
        s3: PB3<Input<Floating>>,
        s4: PC3<Input<Floating>>,
        s5: PD4<Input<Floating>>,
    ) -> ZumoSensors {
        ZumoSensors {
            s0: Some(s0),
            s1: Some(s1),
            s2: Some(s2),
            s3: Some(s3),
            s4: Some(s4),
            s5: Some(s5),
        }
    }
}

impl Uno {
    pub fn read_sensors(&mut self) -> [u32; 6] {
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

        let s0 = s0.into_floating_input(&mut self.ddr);
        let s1 = s1.into_floating_input(&mut self.ddr);
        let s2 = s2.into_floating_input(&mut self.ddr);
        let s3 = s3.into_floating_input(&mut self.ddr);
        let s4 = s4.into_floating_input(&mut self.ddr);
        let s5 = s5.into_floating_input(&mut self.ddr);

        let start_time = timers::micros();
        let stop_time = start_time + SENSOR_TIMEOUT_US;
        let mut sensor_values = [SENSOR_TIMEOUT_US; 6];
        loop {
            let time = timers::micros() - start_time;
            if time >= stop_time {
                break;
            }

            if s0.is_low().void_unwrap() && time < sensor_values[0] {
                sensor_values[0] = time;
            }
            if s1.is_low().void_unwrap() && time < sensor_values[1] {
                sensor_values[1] = time;
            }
            if s2.is_low().void_unwrap() && time < sensor_values[2] {
                sensor_values[2] = time;
            }
            if s3.is_low().void_unwrap() && time < sensor_values[3] {
                sensor_values[3] = time;
            }
            if s4.is_low().void_unwrap() && time < sensor_values[4] {
                sensor_values[4] = time;
            }
            if s5.is_low().void_unwrap() && time < sensor_values[5] {
                sensor_values[5] = time;
            }
        }

        self.sensors.s0 = Some(s0);
        self.sensors.s1 = Some(s1);
        self.sensors.s2 = Some(s2);
        self.sensors.s3 = Some(s3);
        self.sensors.s4 = Some(s4);
        self.sensors.s5 = Some(s5);

        return sensor_values;
    }
}
