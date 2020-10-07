use embedded_hal::{
    digital::v2::OutputPin,
    PwmPin,
};
use void::{
    ResultVoidExt,
    Void,
};

const MAX_MOTOR_DELTA: f32 = 0.1; // 10% of full power
pub const UPDATE_DELAY_US: u32 = 10000;

enum MotorDirection {
    Forward,
    Reverse,
}

pub struct MotorController<PD: OutputPin, PT: PwmPin> {
    direction_pin: PD,
    throttle_pin: PT,
    pub target_value: f32,
    pub current_value: f32,
    last_update_time: u32,
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
            target_value: 0.0,
            current_value: 0.0,
            last_update_time: 0,
        }
    }

    pub fn set(&mut self, value: f32) {
        self.target_value = value;
    }

    pub fn update(&mut self, now: u32) {
        if now < self.last_update_time + UPDATE_DELAY_US || self.current_value == self.target_value {
            return;
        }

        self.current_value = match self.current_value {
            cv if cv < self.target_value - MAX_MOTOR_DELTA => cv + MAX_MOTOR_DELTA,
            cv if cv > self.target_value + MAX_MOTOR_DELTA => cv - MAX_MOTOR_DELTA,
            _ => self.target_value,
        };

        let (dir, throttle) = compute_direction_and_throttle(self.current_value);
        match dir {
            MotorDirection::Forward => self.direction_pin.set_low().void_unwrap(),
            MotorDirection::Reverse => self.direction_pin.set_high().void_unwrap(),
        }
        self.throttle_pin.set_duty(throttle);
    }
}

fn compute_direction_and_throttle(value: f32) -> (MotorDirection, u8) {
    // constrain value between [-1, 1]
    let value = match value {
        v if v < -1.0 => -1.0,
        v if v > 1.0 => 1.0,
        _ => value,
    };

    let direction = match value {
        v if v < 0.0 => MotorDirection::Reverse,
        _ => MotorDirection::Forward,
    };

    let throttle = if value < 0.0 { value * -255.0 } else { value * 255.0 };
    return (direction, throttle as u8);
}