//! Configuration persistence layer

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Micro break interval in minutes
    pub micro_break_interval_minutes: u32,
    /// Micro break duration in seconds
    pub micro_break_duration_seconds: u32,
    /// Long break interval in minutes
    pub long_break_interval_minutes: u32,
    /// Long break duration in minutes
    pub long_break_duration_minutes: u32,
    /// Auto-start on system boot
    pub auto_start: bool,
    /// Whether breaks are enabled
    pub enabled: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            micro_break_interval_minutes: 20,
            micro_break_duration_seconds: 20,
            long_break_interval_minutes: 60,
            long_break_duration_minutes: 5,
            auto_start: true,
            enabled: true,
        }
    }
}

impl Config {
    /// Get the path to the config file
    fn config_path() -> Result<PathBuf, io::Error> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?;

        let app_config_dir = config_dir.join("lookout");

        // Create directory if it doesn't exist
        fs::create_dir_all(&app_config_dir)?;

        Ok(app_config_dir.join("config.json"))
    }

    /// Load configuration from disk, or create default if not found
    pub fn load() -> Result<Self, io::Error> {
        let path = Self::config_path()?;

        if path.exists() {
            let contents = fs::read_to_string(&path)?;
            serde_json::from_str(&contents).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Failed to parse config: {e}"),
                )
            })
        } else {
            log::info!("Config file not found, creating default");
            let config = Self::default();
            config.save()?;
            Ok(config)
        }
    }

    /// Save configuration to disk
    pub fn save(&self) -> Result<(), io::Error> {
        let path = Self::config_path()?;
        let contents = serde_json::to_string_pretty(self).map_err(|e| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("Failed to serialize config: {e}"),
            )
        })?;

        fs::write(&path, contents)?;
        log::debug!("Config saved to {}", path.display());
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.micro_break_interval_minutes, 20);
        assert_eq!(config.micro_break_duration_seconds, 20);
        assert_eq!(config.long_break_interval_minutes, 60);
        assert_eq!(config.long_break_duration_minutes, 5);
        assert!(config.auto_start);
        assert!(config.enabled);
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let json = serde_json::to_string(&config).expect("Failed to serialize");
        let deserialized: Config = serde_json::from_str(&json).expect("Failed to deserialize");

        assert_eq!(
            config.micro_break_interval_minutes,
            deserialized.micro_break_interval_minutes
        );
        assert_eq!(config.enabled, deserialized.enabled);
    }
}
