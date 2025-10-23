//! Break timer engine with tokio async timers

use crate::config::Config;
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock};
use tokio::time::{interval, Duration, Instant};

/// Type of break
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakType {
    Micro,
    Long,
}

/// Timer state
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)]
pub enum TimerState {
    Running,
    Paused,
    InBreak(BreakType),
}

/// Break event emitted by the timer
#[derive(Debug, Clone)]
pub enum BreakEvent {
    BreakStarted(BreakType, Duration),
    BreakEnded(BreakType),
}

/// Break timer engine
pub struct TimerEngine {
    config: Config,
    state: Arc<RwLock<TimerState>>,
    next_micro_break: Arc<RwLock<Instant>>,
    next_long_break: Arc<RwLock<Instant>>,
    event_sender: broadcast::Sender<BreakEvent>,
}

impl TimerEngine {
    /// Create a new timer engine
    pub fn new(config: Config) -> Self {
        let now = Instant::now();
        let (event_sender, _) = broadcast::channel(16);

        let micro_interval =
            Duration::from_secs(u64::from(config.micro_break_interval_minutes) * 60);
        let long_interval = Duration::from_secs(u64::from(config.long_break_interval_minutes) * 60);

        Self {
            config,
            state: Arc::new(RwLock::new(TimerState::Running)),
            next_micro_break: Arc::new(RwLock::new(now + micro_interval)),
            next_long_break: Arc::new(RwLock::new(now + long_interval)),
            event_sender,
        }
    }

    /// Subscribe to break events
    pub fn subscribe(&self) -> broadcast::Receiver<BreakEvent> {
        self.event_sender.subscribe()
    }

    /// Start the timer engine
    pub async fn start(&self) {
        log::info!("Timer engine started");

        let mut tick_interval = interval(Duration::from_secs(1));

        loop {
            tick_interval.tick().await;

            let state = *self.state.read().await;

            // Skip processing if paused or in break
            if state != TimerState::Running {
                continue;
            }

            let now = Instant::now();

            // Check for micro break
            let micro_time = *self.next_micro_break.read().await;
            if now >= micro_time {
                log::info!("Micro break time!");
                self.trigger_break(BreakType::Micro).await;
            }

            // Check for long break
            let long_time = *self.next_long_break.read().await;
            if now >= long_time {
                log::info!("Long break time!");
                self.trigger_break(BreakType::Long).await;
            }
        }
    }

    /// Trigger a break
    pub async fn trigger_break(&self, break_type: BreakType) {
        let duration = match break_type {
            BreakType::Micro => {
                Duration::from_secs(u64::from(self.config.micro_break_duration_seconds))
            },
            BreakType::Long => {
                Duration::from_secs(u64::from(self.config.long_break_duration_minutes) * 60)
            },
        };

        // Update state
        {
            let mut state = self.state.write().await;
            *state = TimerState::InBreak(break_type);
        }

        // Emit event
        let _ = self
            .event_sender
            .send(BreakEvent::BreakStarted(break_type, duration));

        // Wait for break duration
        tokio::time::sleep(duration).await;

        // End break
        {
            let mut state = self.state.write().await;
            *state = TimerState::Running;
        }

        // Schedule next break
        self.schedule_next_break(break_type).await;

        // Emit end event
        let _ = self.event_sender.send(BreakEvent::BreakEnded(break_type));

        log::info!("{break_type:?} break ended");
    }

    /// Schedule the next break of the given type
    async fn schedule_next_break(&self, break_type: BreakType) {
        let now = Instant::now();

        match break_type {
            BreakType::Micro => {
                let interval =
                    Duration::from_secs(u64::from(self.config.micro_break_interval_minutes) * 60);
                *self.next_micro_break.write().await = now + interval;
                log::debug!("Next micro break scheduled in {interval:?}");
            },
            BreakType::Long => {
                let interval =
                    Duration::from_secs(u64::from(self.config.long_break_interval_minutes) * 60);
                *self.next_long_break.write().await = now + interval;
                log::debug!("Next long break scheduled in {interval:?}");
            },
        }
    }

    /// Pause the timer
    #[allow(dead_code)]
    pub async fn pause(&self) {
        let mut state = self.state.write().await;
        *state = TimerState::Paused;
        log::info!("Timer paused");
    }

    /// Resume the timer
    #[allow(dead_code)]
    pub async fn resume(&self) {
        let mut state = self.state.write().await;
        *state = TimerState::Running;
        log::info!("Timer resumed");
    }

    /// Get current state
    #[allow(dead_code)]
    pub async fn state(&self) -> TimerState {
        *self.state.read().await
    }

    /// Get time until next micro break
    pub async fn time_until_micro_break(&self) -> Duration {
        let next = *self.next_micro_break.read().await;
        next.saturating_duration_since(Instant::now())
    }

    /// Get time until next long break
    pub async fn time_until_long_break(&self) -> Duration {
        let next = *self.next_long_break.read().await;
        next.saturating_duration_since(Instant::now())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_timer_creation() {
        let config = Config::default();

        let timer = TimerEngine::new(config);
        assert_eq!(timer.state().await, TimerState::Running);
    }

    #[tokio::test]
    async fn test_timer_pause_resume() {
        let config = Config::default();
        let timer = TimerEngine::new(config);

        timer.pause().await;
        assert_eq!(timer.state().await, TimerState::Paused);

        timer.resume().await;
        assert_eq!(timer.state().await, TimerState::Running);
    }

    #[tokio::test]
    async fn test_event_subscription() {
        let config = Config::default();
        let timer = TimerEngine::new(config);

        let mut receiver = timer.subscribe();

        // Trigger a break manually for testing
        let duration = Duration::from_secs(1);
        let _ = timer
            .event_sender
            .send(BreakEvent::BreakStarted(BreakType::Micro, duration));

        let event = receiver.recv().await;
        assert!(event.is_ok());
    }
}
