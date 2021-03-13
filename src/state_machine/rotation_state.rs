use crate::{
    state_machine,
    state_machine::State,
    uno::{
        motor,
        Uno,
    },
};
use avr_async::Waiter;
use micromath::F32Ext;

const TOLERANCE: f32 = 5.0;
const BASE_SPEED: f32 = 0.0;
const ROTATION_UPDATE_MS: u32 = 20; // The IMU's output data rate is 50Hz, or 1 / 20ms.

fn degrees_delta(heading_from: f32, heading_to: f32) -> f32 {
    let mut delta = heading_to - heading_from;
    if delta > 180.0 {
        delta -= 360.0;
    } else if delta < -180.0 {
        delta += 360.0;
    }

    delta
}

pub async fn rotation_future(uno: &mut Uno, angle: f32) -> State {
    // Turn off the motors to reduce interference
    uno.motor_controller.set_targets(0.0, 0.0);
    Waiter::new(500).await; // It takes ~400ms for the motors to fully stop

    let mut new_heading = uno.imu.get_current_heading_degrees() + angle;
    if new_heading > 360.0 {
        new_heading -= 360.0;
    }

    loop {
        let delta = degrees_delta(uno.imu.get_current_heading_degrees(), new_heading);
        if delta <= TOLERANCE {
            uno.motor_controller.set_targets(0.0, 0.0);
            Waiter::new(100).await;
            break;
        }

        let mut speed = 0.6 * delta / 180.0;
        if speed < 0.0 {
            speed -= BASE_SPEED;
        } else {
            speed += BASE_SPEED;
        }

        uno.motor_controller.set_targets(speed, -speed);
        Waiter::new(ROTATION_UPDATE_MS).await;
    }

    return State::Exploration { found_edge: false };
}
