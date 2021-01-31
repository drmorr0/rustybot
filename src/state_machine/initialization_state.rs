use crate::{
    avr_async::Waiter,
    state_machine::State,
    uno::{
        timers,
        MotorController,
        Uno,
    },
};
use arduino_uno::prelude::*;

const CONFIG_EXTRA_PRESSES: u8 = 2;

pub async fn initialization_future(uno: &mut Uno) -> State {
    uno.pushbutton.wait_for_press().await;
    let additional_button_presses = uno.pushbutton.count_presses_before(timers::millis() + 1000).await;
    if additional_button_presses >= CONFIG_EXTRA_PRESSES {
        State::Calibration
    } else {
        uno.load_calibration_data().await;
        State::Exploration { found_edge: false }
    }
}
