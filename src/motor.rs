use embedded_hal::{
    digital::v2::OutputPin,
    PwmPin,
};
use void::{
    ResultVoidExt,
    Void,
};

enum MotorDirection {
    Forward,
    Reverse,
}

pub struct MotorController<PD: OutputPin, PT: PwmPin> {
    direction_pin: PD,
    throttle_pin: PT,
}

impl<PD: OutputPin, PT: PwmPin> MotorController<PD, PT>
where
    PD: OutputPin<Error = Void>,
    PT: PwmPin<Duty = u8>,
{
    pub fn new(direction_pin: PD, mut throttle_pin: PT) -> MotorController<PD, PT> {
        throttle_pin.enable();
        MotorController {
            direction_pin,
            throttle_pin,
        }
    }

    pub fn set(&mut self, value: f32) {
        let (dir, throttle) = compute_direction_and_throttle(value);
        match dir {
            MotorDirection::Forward => self.direction_pin.set_low().void_unwrap(),
            MotorDirection::Reverse => self.direction_pin.set_high().void_unwrap(),
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
