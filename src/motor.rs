use arduino_uno::{
    hal::{
        port::{
            mode::*,
            portb::*,
            portd::*,
        },
        pwm::Timer1Pwm,
    },
    prelude::*,
    Pins,
    DDR,
};

enum MotorDirection {
    Forward,
    Reverse,
}

pub struct LeftMotorController {
    direction_pin: PB0<Output>,
    throttle_pin: PB2<Pwm<Timer1Pwm>>,
}

pub struct RightMotorController {
    direction_pin: PD7<Output>,
    throttle_pin: PB1<Pwm<Timer1Pwm>>,
}

impl LeftMotorController {
    pub fn new(
        direction_pin: PB0<Input<Floating>>,
        throttle_pin: PB2<Input<Floating>>,
        ddr: &DDR,
        pwm_timer: &mut Timer1Pwm,
    ) -> LeftMotorController {
        let mut throttle_pin = throttle_pin.into_output(ddr).into_pwm(pwm_timer);
        throttle_pin.enable();
        LeftMotorController {
            direction_pin: direction_pin.into_output(ddr),
            throttle_pin,
        }
    }

    pub fn set(&mut self, value: f32) {
        let (dir, throttle) = compute_direction_and_throttle(value);
        match dir {
            MotorDirection::Forward => self.direction_pin.set_high().void_unwrap(),
            MotorDirection::Reverse => self.direction_pin.set_low().void_unwrap(),
        }
        self.throttle_pin.set_duty(throttle);
    }
}

impl RightMotorController {
    pub fn new(
        direction_pin: PD7<Input<Floating>>,
        throttle_pin: PB1<Input<Floating>>,
        ddr: &DDR,
        pwm_timer: &mut Timer1Pwm,
    ) -> RightMotorController {
        let mut throttle_pin = throttle_pin.into_output(ddr).into_pwm(pwm_timer);
        throttle_pin.enable();
        RightMotorController {
            direction_pin: direction_pin.into_output(ddr),
            throttle_pin,
        }
    }

    pub fn set(&mut self, value: f32) {
        let (dir, throttle) = compute_direction_and_throttle(value);
        match dir {
            MotorDirection::Forward => self.direction_pin.set_high().void_unwrap(),
            MotorDirection::Reverse => self.direction_pin.set_low().void_unwrap(),
        }
        self.throttle_pin.set_duty(throttle);
    }
}

fn compute_direction_and_throttle(value: f32) -> (MotorDirection, u8) {
    // constrain value between [-1, 1]
    let value = if value < -1.0 {
        -1.0
    } else if value > 1.0 {
        1.0
    } else {
        value
    };

    let direction = if value < 0.0 {
        MotorDirection::Reverse
    } else {
        MotorDirection::Forward
    };

    let throttle = if value < 0.0 { value * -255.0 } else { value * 255.0 };
    return (direction, throttle as u8);
}
