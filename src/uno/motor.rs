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

type LeftDirectionPin = PB0<Output>;
type LeftThrottlePin = PB2<Pwm<pwm::Timer1Pwm>>;
type RightDirectionPin = PD7<Output>;
type RightThrottlePin = PB1<Pwm<pwm::Timer1Pwm>>;

const MAX_MOTOR_DELTA: f32 = 0.1; // 10% of full power
const UPDATE_DELAY_MS: u32 = 10;
const BORROW_MUT_DELAY_MS: u32 = 5;

enum MotorDirection {
    Forward,
    Reverse,
}

struct SingleMotorController<PD, PT>
where
    PD: 'static + OutputPin<Error = Void>,
    PT: 'static + PwmPin<Duty = u8>,
{
    direction_pin: PD,
    throttle_pin: PT,
    current_value: f32,
}

impl<PD, PT> SingleMotorController<PD, PT>
where
    PD: 'static + OutputPin<Error = Void>,
    PT: 'static + PwmPin<Duty = u8>,
{
    fn update(&mut self, target_value: f32) {
        self.current_value = match self.current_value {
            cv if cv < target_value - MAX_MOTOR_DELTA => cv + MAX_MOTOR_DELTA,
            cv if cv > target_value + MAX_MOTOR_DELTA => cv - MAX_MOTOR_DELTA,
            _ => target_value,
        };

        let (dir, throttle) = compute_direction_and_throttle(self.current_value);
        match dir {
            MotorDirection::Forward => self.direction_pin.set_low().void_unwrap(),
            MotorDirection::Reverse => self.direction_pin.set_high().void_unwrap(),
        }
        self.throttle_pin.set_duty(throttle);
    }
}

pub struct MotorController {
    left: RefCell<SingleMotorController<LeftDirectionPin, LeftThrottlePin>>,
    left_target: RefCell<f32>,
    right: RefCell<SingleMotorController<RightDirectionPin, RightThrottlePin>>,
    right_target: RefCell<f32>,
}

impl MotorController {
    // We have to construct these as static references, because the driver (future) holds
    // a reference to "self" that is expected to be static
    pub fn new(
        left_direction_pin: LeftDirectionPin,
        mut left_throttle_pin: LeftThrottlePin,
        right_direction_pin: RightDirectionPin,
        mut right_throttle_pin: RightThrottlePin,
    ) -> &'static MotorController {
        left_throttle_pin.enable();
        right_throttle_pin.enable();

        Allocator::get().new(MotorController {
            left: RefCell::new(SingleMotorController {
                direction_pin: left_direction_pin,
                throttle_pin: left_throttle_pin,
                current_value: 0.0,
            }),
            left_target: RefCell::new(0.0),
            right: RefCell::new(SingleMotorController {
                direction_pin: right_direction_pin,
                throttle_pin: right_throttle_pin,
                current_value: 0.0,
            }),
            right_target: RefCell::new(0.0),
        })
    }

    pub fn set_targets(&self, left_target: f32, right_target: f32) {
        if let Ok(mut lt) = self.left_target.try_borrow_mut() {
            *lt = left_target;
        }
        if let Ok(mut rt) = self.right_target.try_borrow_mut() {
            *rt = right_target;
        }
    }

    pub fn scale_targets(&self, scalar: f32) {
        if let Ok(mut lt) = self.left_target.try_borrow_mut() {
            *lt *= scalar;
        }
        if let Ok(mut rt) = self.right_target.try_borrow_mut() {
            *rt *= scalar;
        }
    }

    pub fn get_motor_driver(&'static self) -> &'static mut dyn Future<Output = !> {
        let future = async move || loop {
            if let Ok(mut left) = self.left.try_borrow_mut() {
                match self.left_target.try_borrow() {
                    Ok(lt) if *lt != left.current_value => left.update(*lt),
                    _ => (),
                }
            }
            if let Ok(mut right) = self.right.try_borrow_mut() {
                match self.right_target.try_borrow() {
                    Ok(rt) if *rt != right.current_value => right.update(*rt),
                    _ => (),
                }
            }
            Waiter::new(UPDATE_DELAY_MS).await;
        };
        Allocator::get().new(future())
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
