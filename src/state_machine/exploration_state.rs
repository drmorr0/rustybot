use crate::{
    avr_async::Waiter,
    state_machine::State,
    uno::{
        MotorController,
        Uno,
    },
};

pub async fn exploration_future(uno: &mut Uno, found_edge: bool) -> State {
    uno.read_ir_sensor_values().await;
    let triggered_count = uno.ir_sensors.values.iter().filter(|&&x| x > 1500).count();

    let mut wait_time_ms: u32 = 100;
    let mut state = State::Exploration { found_edge: false };
    if triggered_count > 1 {
        wait_time_ms = 0;
        if found_edge {
            state = State::Exploration { found_edge: true };
        } else {
            state = State::BoundaryDetected;
        }
    }

    Waiter::new(wait_time_ms).await;
    return state;
}
