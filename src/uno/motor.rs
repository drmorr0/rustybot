use crate::{
    avr_async::Waiter,
    mem::Allocator,
};
use arduino_uno::hal::{
    port::{
        mode::*,
        portb::*,
        portd::*,
    },
    pwm,
};
use core::{
    cell::RefCell,
    future::Future,
};
use embedded_hal::{
    digital::v2::OutputPin,
    PwmPin,
};
use void::{
    ResultVoidExt,
    Void,
};

const MAX_MOTOR_DELTA: f32 = 0.1; // 10% of full power
const UPDATE_DELAY_MS: u32 = 40;

enum MotorDirection {
    Forward,
    Reverse,
}

pub struct MotorController {
    pub left_target: f32,
    pub right_target: f32,
}

impl MotorController {
    // We have to construct these as static references, because the driver (future) holds
    // a reference to "self" that is expected to be static
    pub fn new() -> &'static mut RefCell<MotorController> {
        Allocator::get().new(RefCell::new(MotorController {
            left_target: 0.0,
            right_target: 0.0,
        }))
    }
}

pub fn get_motor_driver(
    controller_ref: &'static RefCell<MotorController>,
    mut left_direction_pin: PB0<Output>,
    mut left_throttle_pin: PB2<Pwm<pwm::Timer1Pwm>>,
    mut right_direction_pin: PD7<Output>,
    mut right_throttle_pin: PB1<Pwm<pwm::Timer1Pwm>>,
) -> &'static mut dyn Future<Output = !> {
    left_throttle_pin.enable();
    right_throttle_pin.enable();
    let mut current_left_value: f32 = 0.0;
    let mut current_right_value: f32 = 0.0;
    let future = async move || loop {
        if let Ok(controller) = controller_ref.try_borrow() {
            if current_left_value != controller.left_target {
                update_motor(
                    controller.left_target,
                    &mut current_left_value,
                    &mut left_direction_pin,
                    &mut left_throttle_pin,
                );
            }

            if current_right_value != controller.right_target {
                update_motor(
                    controller.right_target,
                    &mut current_right_value,
                    &mut right_direction_pin,
                    &mut right_throttle_pin,
                );
            }
        }
        Waiter::new(UPDATE_DELAY_MS).await;
    };
    Allocator::get().new(future())
}

fn update_motor<PD: 'static + OutputPin<Error = Void>, PT: 'static + PwmPin<Duty = u8>>(
    target_value: f32,
    current_value: &mut f32,
    direction_pin: &mut PD,
    throttle_pin: &mut PT,
) {
    *current_value = match *current_value {
        cv if cv < target_value - MAX_MOTOR_DELTA => cv + MAX_MOTOR_DELTA,
        cv if cv > target_value + MAX_MOTOR_DELTA => cv - MAX_MOTOR_DELTA,
        _ => target_value,
    };

    let (dir, throttle) = compute_direction_and_throttle(*current_value);
    match dir {
        MotorDirection::Forward => direction_pin.set_low().void_unwrap(),
        MotorDirection::Reverse => direction_pin.set_high().void_unwrap(),
    }
    throttle_pin.set_duty(throttle);
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
