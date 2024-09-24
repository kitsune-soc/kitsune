#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use std::{
    cell::RefCell,
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Weak,
    },
    time::{Duration, SystemTime},
};

thread_local! {
    /// Thread-local clock
    ///
    /// Defaults to a default [`Clock`], not to a `None` since clocks are cheap to instantiate
    static THREAD_CLOCK: RefCell<Clock> = RefCell::new(Clock::default());
}

/// Duration the delta should be adjusted in
#[derive(Clone, Copy, PartialEq)]
pub enum DeltaDirection {
    /// Add to the delta
    Add,

    /// Subtract from the delta
    Sub,
}

/// Handle to adjust the delta of the clock
#[derive(Clone)]
pub struct MockHandle {
    delta: Weak<AtomicI64>,
}

impl MockHandle {
    /// Adjust the delta by the duration in the direction specified
    #[inline]
    pub fn adjust(&self, direction: DeltaDirection, delta: Duration) {
        let Some(delta_handle) = self.delta.upgrade() else {
            return;
        };

        let mut delta = delta.as_nanos() as i64;
        if direction == DeltaDirection::Sub {
            delta = -delta;
        }

        delta_handle.fetch_add(delta, Ordering::AcqRel);
    }

    /// Reset the offset to 0
    #[inline]
    pub fn reset(&self) {
        if let Some(delta_handle) = self.delta.upgrade() {
            delta_handle.store(0, Ordering::Release);
        }
    }
}

/// Guard which will reset the thread-local upon drop
pub struct ClockGuard {
    old_clock: Option<Clock>,
}

impl Drop for ClockGuard {
    fn drop(&mut self) {
        THREAD_CLOCK.with(|clock| clock.replace(self.old_clock.take().unwrap()));
    }
}

/// Clock with an optional adjustable delta
#[derive(Clone, Default)]
pub struct Clock {
    delta: Option<Arc<AtomicI64>>,
}

impl Clock {
    /// Construct a new clock without an internal delta
    #[inline]
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Construct a mockable clock
    ///
    /// This clock returns a handle which you can use to adjust the delta
    #[inline]
    #[must_use]
    pub fn mockable() -> (Self, MockHandle) {
        let delta = Arc::new(AtomicI64::default());

        let mock_handle = MockHandle {
            delta: Arc::downgrade(&delta),
        };
        let clock = Self { delta: Some(delta) };

        (clock, mock_handle)
    }

    /// Enter a context where this clock is installed into the thread-local context
    ///
    /// As long as the guard is kept live, the [`now`] function will read the time of this clock
    #[inline]
    #[must_use]
    pub fn enter(&self) -> ClockGuard {
        let old_clock = THREAD_CLOCK.with(|clock| clock.replace(self.clone()));

        ClockGuard {
            old_clock: Some(old_clock),
        }
    }

    /// Read the current time from the system clock and apply the delta
    #[inline]
    #[must_use]
    pub fn now(&self) -> SystemTime {
        let mut now = SystemTime::now();

        if let Some(ref delta) = self.delta {
            let ns_delta = delta.load(Ordering::Acquire);
            if ns_delta.is_positive() {
                now += Duration::from_nanos(ns_delta as u64);
            } else {
                now -= Duration::from_nanos(ns_delta.unsigned_abs());
            }
        }

        now
    }
}

/// Read the current time from the thread-local clock
#[inline]
#[must_use]
pub fn now() -> SystemTime {
    THREAD_CLOCK.with(|clock| clock.borrow().now())
}

#[cfg(test)]
mod test {
    use crate::{Clock, DeltaDirection};
    use std::time::Duration;

    #[test]
    fn can_forward() {
        let (clock, mock) = Clock::mockable();
        let _clock_guard = clock.enter();

        let now = crate::now();
        mock.adjust(DeltaDirection::Add, Duration::from_secs(1));
        let after = crate::now();

        let delta = after.duration_since(now).unwrap();
        assert_eq!(delta.as_secs_f32().round() as u8, 1);
    }

    #[test]
    fn can_rewind() {
        let (clock, mock) = Clock::mockable();
        let _clock_guard = clock.enter();

        let now = crate::now();
        mock.adjust(DeltaDirection::Sub, Duration::from_secs(1));
        let after = crate::now();

        let delta = now.duration_since(after).unwrap();
        assert_eq!(delta.as_secs_f32().round() as u8, 1);
    }

    #[test]
    fn can_reset() {
        let (clock, mock) = Clock::mockable();
        let _clock_guard = clock.enter();

        let now = crate::now();
        mock.adjust(DeltaDirection::Add, Duration::from_secs(10));
        let adjusted = crate::now();
        mock.reset();
        let reset = crate::now();

        // "adjusted" should be ahead of "now" and "reset"
        assert!(now < adjusted && reset < adjusted);

        // "now" and "reset" should be pretty much equal
        //
        // Technically this very much depends on the system clock and execution speed,
        // but it should pretty much always hold true
        assert_eq!(reset.duration_since(now).unwrap().as_secs(), 0);
    }
}
