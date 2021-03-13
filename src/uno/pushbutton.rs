use crate::{
    mem::Allocator,
    state_machine::UPDATE_DELAY_MS,
};
use arduino_uno::{
    hal::port::{
        mode::*,
        portb::*,
    },
    prelude::*,
};
use avr_async::{
    millis,
    Waiter,
};

const DEBOUNCE_MS: u32 = 10;

type ButtonInput = PB4<Input<PullUp>>;

pub enum ButtonState {
    Pressed,
    Released,
}

pub struct Pushbutton {
    pin: ButtonInput,
}

impl Pushbutton {
    pub fn new(pin: ButtonInput) -> Pushbutton {
        Pushbutton { pin }
    }

    pub async fn count_presses_before(&self, end_time_ms: u32) -> u8 {
        let mut count = 0;
        while millis() <= end_time_ms {
            if self.wait_for_press_before(Some(end_time_ms)).await {
                count += 1;
            }
        }
        count
    }

    pub async fn wait_for_press(&self) {
        self.wait_for_press_before(None).await;
    }

    pub async fn wait_for_press_before(&self, end_time_ms: Option<u32>) -> bool {
        let mut triggered = self.wait_for(ButtonState::Pressed, end_time_ms).await;
        triggered &= self.wait_for(ButtonState::Released, end_time_ms).await;
        triggered
    }

    pub async fn wait_for(&self, state: ButtonState, end_time_ms: Option<u32>) -> bool {
        let check_fn = match state {
            ButtonState::Pressed => ButtonInput::is_low,
            ButtonState::Released => ButtonInput::is_high,
        };

        let mut triggered = false;
        while millis() <= end_time_ms.unwrap_or(u32::MAX) {
            if !check_fn(&self.pin).void_unwrap() {
                Waiter::new(UPDATE_DELAY_MS).await;
                continue;
            }
            Waiter::new(DEBOUNCE_MS).await;
            if check_fn(&self.pin).void_unwrap() {
                triggered = true;
                break;
            }
        }
        triggered
    }
}
