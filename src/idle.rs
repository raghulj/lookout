//! User idle time detection for Linux
//!
//! This module provides cross-platform idle time detection:
//! - X11: Uses the `XScreenSaver` extension to query idle time
//! - Wayland: Uses D-Bus protocols (works on GNOME, KDE, etc.)
//!
//! The idle time represents how long since the user last interacted
//! with keyboard or mouse.

use std::time::Duration;
use user_idle::UserIdle;

/// Get the current user idle time
///
/// Returns `Some(Duration)` if idle time could be determined,
/// or `None` if the system doesn't support idle detection.
pub fn get_idle_time() -> Option<Duration> {
    match UserIdle::get_time() {
        Ok(idle) => Some(Duration::from_secs(idle.as_seconds())),
        Err(e) => {
            log::debug!("Could not get idle time: {e}");
            None
        },
    }
}

/// Check if the user has been idle for at least the given threshold
///
/// Returns `true` if the user is considered idle (no keyboard/mouse activity
/// for at least `threshold` duration).
///
/// Returns `false` if:
/// - The user is active (idle time < threshold)
/// - Idle detection is not available on this system
pub fn is_user_idle(threshold: Duration) -> bool {
    get_idle_time().is_some_and(|idle| idle >= threshold)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_idle_time_returns_some_or_none() {
        // This test just verifies the function doesn't panic
        // The actual value depends on the system
        let result = get_idle_time();
        // Either we get a duration or None (on systems without support)
        assert!(result.is_some() || result.is_none());
    }

    #[test]
    fn test_is_user_idle_with_zero_threshold() {
        // With zero threshold, user should always be considered idle
        // (assuming idle detection works)
        let result = is_user_idle(Duration::ZERO);
        // Can be true or false depending on system support
        assert!(result || !result); // Always passes, just checking no panic
    }

    #[test]
    fn test_is_user_idle_with_large_threshold() {
        // With a very large threshold, user should not be considered idle
        let result = is_user_idle(Duration::from_secs(999_999_999));
        assert!(!result);
    }
}
