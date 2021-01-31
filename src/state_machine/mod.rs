mod calibration_state;
mod exploration_state;
mod initialization_state;
mod rotation_state;

use self::{
    calibration_state::calibration_future,
    exploration_state::exploration_future,
    initialization_state::initialization_future,
    rotation_state::rotation_future,
};
use crate::{
    mem::Allocator,
    uno::MotorController,
    Uno,
};
use core::{
    cell::RefCell,
    future::Future,
};

pub const UPDATE_DELAY_MS: u32 = 100;

pub enum State {
    Calibration,
    Exploration { found_edge: bool },
    Initialization,
    Rotation { angle: f32 },
}

pub fn build_state_machine(uno: &'static mut Uno) -> &'static mut dyn Future<Output = !> {
    let mut current_state = State::Initialization;
    let future = async move || loop {
        current_state = match current_state {
            State::Calibration => calibration_future(uno).await,
            State::Exploration { found_edge } => exploration_future(uno, found_edge).await,
            State::Initialization => initialization_future(uno).await,
            State::Rotation { angle } => rotation_future(uno, angle).await,
        };
    };
    Allocator::get().new(future())
}
