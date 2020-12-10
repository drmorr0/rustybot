use crate::{
    avr_async::Waiter,
    state_machine::State,
    uno::{
        MotorController,
        Uno,
    },
};

pub async fn boundary_detected_future(uno: &mut Uno) -> State {
    let mut wait_time_ms: u32 = 100;
    let mut state = State::BoundaryDetected;
    if let Ok(mut mc) = uno.motor_controller.try_borrow_mut() {
        mc.left_target *= -1.0;
        mc.right_target *= -1.0;
        state = State::Exploration { found_edge: true };
    } else {
        wait_time_ms = 5;
    }

    Waiter::new(wait_time_ms).await;
    return state;
}
