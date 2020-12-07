mod boundary_detected_state;
mod exploration_state;

use self::{
    boundary_detected_state::boundary_detected_future,
    exploration_state::exploration_future,
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

pub enum State {
    Exploration,
    BoundaryDetected,
}

pub fn build_state_machine(uno: &'static mut Uno) -> &'static mut dyn Future<Output = !> {
    let mut current_state = State::Exploration;
    let future = async move || loop {
        current_state = match current_state {
            State::Exploration => exploration_future(uno).await,
            State::BoundaryDetected => boundary_detected_future(uno).await,
        };
    };
    Allocator::get().new(future())
}
