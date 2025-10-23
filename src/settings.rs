//! Settings management layer

use crate::config::Config;
use std::sync::{Arc, RwLock};
use tokio::sync::broadcast;

/// Thread-safe settings manager
#[derive(Clone)]
pub struct Settings {
    config: Arc<RwLock<Config>>,
    update_sender: broadcast::Sender<Config>,
}

impl Settings {
    /// Create a new settings manager
    pub fn new() -> Self {
        let config = Config::load().unwrap_or_else(|e| {
            log::warn!("Failed to load config: {e}, using defaults");
            Config::default()
        });

        let (update_sender, _) = broadcast::channel(16);

        Self {
            config: Arc::new(RwLock::new(config)),
            update_sender,
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
        let updated_config = {
            let mut config = self
                .config
                .write()
                .map_err(|e| format!("Failed to acquire write lock: {e}"))?;

            f(&mut config);

            config
                .save()
                .map_err(|e| format!("Failed to save config: {e}"))?;

            config.clone()
        };

        // Notify subscribers of config change
        let _ = self.update_sender.send(updated_config);

        log::info!("Settings updated successfully");
        Ok(())
    }

    /// Subscribe to configuration updates
    pub fn subscribe(&self) -> broadcast::Receiver<Config> {
        self.update_sender.subscribe()
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
