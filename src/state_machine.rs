use crate::Uno;

pub enum State {
    Exploration(ExplorationState),
    BoundaryDetected(BoundaryDetectedState),
}

pub trait StateObject {
    fn update(&mut self, uno: &mut Uno, current_time_us: u32) -> Option<State>;
}

impl StateObject for State {
    fn update(&mut self, uno: &mut Uno, current_time_us: u32) -> Option<State> {
        match self {
            State::Exploration(s) => s.update(uno, current_time_us),
            State::BoundaryDetected(s) => s.update(uno, current_time_us),
        }
    }
}

pub struct ExplorationState {
    next_update_time_us: u32,
    speed: f32,
}

impl ExplorationState {
    pub fn new() -> State {
        State::Exploration(Self {
            next_update_time_us: 0,
            speed: -0.5,
        })
    }

    fn update(&mut self, uno: &mut Uno, current_time_us: u32) -> Option<State> {
        if current_time_us >= self.next_update_time_us {
            self.next_update_time_us = current_time_us + 1000000;
            self.speed *= -1.0;
            uno.left_motor.set(self.speed);
            uno.right_motor.set(self.speed);
        }

        // for i in uno.last_sensor_values.iter() {
        //     if *i > 1500 {
        //         return Some(BoundaryDetectedState::new());
        //     }
        // }
        return None;
    }
}

pub struct BoundaryDetectedState {}

impl BoundaryDetectedState {
    pub fn new() -> State {
        State::BoundaryDetected(Self {})
    }

    fn update(&mut self, uno: &mut Uno, current_time_us: u32) -> Option<State> {
        uno.left_motor.set(0.0);
        uno.right_motor.set(0.0);
        None
    }
}
