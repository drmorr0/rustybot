use crate::{
    state_machine,
    state_machine::State,
    uno::{
        MotorController,
        Uno,
    },
};
use avr_async::Waiter;

pub async fn exploration_future(uno: &mut Uno, found_edge: bool) -> State {
    if found_edge {
        uno.motor_controller.set_targets(-0.5, -0.5);
    } else {
        uno.motor_controller.set_targets(0.5, 0.5);
    }

    loop {
        uno.ir_sensors.read_calibrated(&mut uno.ddr).await;
        let triggered_count = uno.ir_sensors.values.iter().filter(|&&x| x > 500).count();

        if triggered_count > 1 {
            if !found_edge {
                return State::Exploration { found_edge: true };
            }
        } else if found_edge {
            // we've moved off the boundary, so now we rotate 90 degrees
            return State::Rotation { angle: 90.0 };
        }

        Waiter::new(state_machine::UPDATE_DELAY_MS).await;
    }
}
