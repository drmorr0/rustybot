use core::{
    cmp::Ordering,
    task::Waker,
};

pub struct TimedWaker {
    pub trigger_time_ms: u32,
    pub waker: Waker,
}

impl PartialEq for TimedWaker {
    fn eq(&self, other: &Self) -> bool {
        self.trigger_time_ms == other.trigger_time_ms
    }
}

impl Eq for TimedWaker {}

impl PartialOrd for TimedWaker {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.trigger_time_ms.cmp(&other.trigger_time_ms))
    }
}

impl Ord for TimedWaker {
    fn cmp(&self, other: &Self) -> Ordering {
        self.trigger_time_ms.cmp(&other.trigger_time_ms)
    }
}
