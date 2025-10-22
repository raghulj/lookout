//! Settings management layer

use crate::config::Config;
use std::sync::{Arc, RwLock};

/// Thread-safe settings manager
#[derive(Clone)]
pub struct Settings {
    config: Arc<RwLock<Config>>,
}

impl Settings {
    /// Create a new settings manager
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_else(|e| {
            log::warn!("Failed to load config: {e}, using defaults");
            Config::default()
        });

        Self {
            config: Arc::new(RwLock::new(config)),
        }
    }

    /// Get a copy of the current configuration
    pub fn get(&self) -> Config {
        self.config
            .read()
            .map(|guard| guard.clone())
            .unwrap_or_default()
    }

    /// Update configuration
    pub fn update<F>(&self, f: F) -> Result<(), String>
    where
        F: FnOnce(&mut Config),
    {
        {
            let mut config = self
                .config
                .write()
                .map_err(|e| format!("Failed to acquire write lock: {e}"))?;

            f(&mut config);

            config
                .save()
                .map_err(|e| format!("Failed to save config: {e}"))?;
        }

        log::info!("Settings updated successfully");
        Ok(())
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
