use crate::{
    avr_async::Waiter,
    state_machine::State,
    uno::{
        MotorController,
        Uno,
    },
};

pub async fn exploration_future(uno: &mut Uno) -> State {
    uno.write_state();
    //let sensors = uno.read_sensors();
    //let triggered_count = sensors.iter().filter(|&&x| x > 1500).count();
    let triggered_count = 0;

    let mut wait_time_ms: u32 = 100;
    let mut state = State::Exploration;
    if triggered_count > 1 {
        wait_time_ms = 0;
        state = State::BoundaryDetected;
    }

    Waiter::new(wait_time_ms).await;
    return state;
}
