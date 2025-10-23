//! Configuration persistence layer

use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::PathBuf;

/// Break message collections
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakMessages {
    /// Main heading messages shown during breaks
    pub headings: Vec<String>,
    /// Instructions for micro breaks
    pub micro_break_instructions: Vec<String>,
    /// Instructions for long breaks
    pub long_break_instructions: Vec<String>,
}

impl Default for BreakMessages {
    fn default() -> Self {
        Self {
            headings: vec![
                "Take a moment".to_string(),
                "Time for a break".to_string(),
                "Pause and breathe".to_string(),
                "Step away".to_string(),
                "Rest your eyes".to_string(),
            ],
            micro_break_instructions: vec![
                "Look at something 20 feet away to reduce eye strain".to_string(),
                "Focus on a distant object and blink slowly".to_string(),
                "Give your eyes a break from the screen".to_string(),
                "Look out the window or across the room".to_string(),
                "Gaze at something far away and relax your eyes".to_string(),
            ],
            long_break_instructions: vec![
                "Stand up, walk around, and give your body a stretch".to_string(),
                "Time to move! Stretch, walk, or grab some water".to_string(),
                "Step away from your desk and move your body".to_string(),
                "Take a walk, stretch your muscles, rest your mind".to_string(),
                "Get up and move around to refresh yourself".to_string(),
            ],
        }
    }
}

impl BreakMessages {
    /// Get a random heading message
    pub fn random_heading(&self) -> &str {
        use rand::seq::SliceRandom;
        self.headings
            .choose(&mut rand::thread_rng())
            .map_or("Take a moment", |s| s.as_str())
    }

    /// Get a random micro break instruction
    pub fn random_micro_instruction(&self) -> &str {
        use rand::seq::SliceRandom;
        self.micro_break_instructions
            .choose(&mut rand::thread_rng())
            .map_or("Look at something distant", |s| s.as_str())
    }

    /// Get a random long break instruction
    pub fn random_long_instruction(&self) -> &str {
        use rand::seq::SliceRandom;
        self.long_break_instructions
            .choose(&mut rand::thread_rng())
            .map_or("Stand up and stretch", |s| s.as_str())
    }
}

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
    /// Background color for break window (CSS rgba format)
    #[serde(default = "default_background_color")]
    pub background_color: String,
    /// Break messages
    #[serde(default)]
    pub break_messages: BreakMessages,
}

fn default_background_color() -> String {
    "rgba(0, 0, 0, 0.95)".to_string()
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
            background_color: default_background_color(),
            break_messages: BreakMessages::default(),
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
