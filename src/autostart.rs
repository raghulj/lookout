//! XDG autostart management for Linux desktop environments

use std::fs;
use std::io;
use std::path::PathBuf;

/// Manages XDG autostart desktop entry
pub struct AutostartManager;

impl AutostartManager {
    /// Get the path to the autostart desktop entry
    fn autostart_path() -> Result<PathBuf, io::Error> {
        let config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Config directory not found"))?;

        let autostart_dir = config_dir.join("autostart");

        // Create autostart directory if it doesn't exist
        fs::create_dir_all(&autostart_dir)?;

        Ok(autostart_dir.join("lookout.desktop"))
    }

    /// Get the path to the current executable
    fn get_executable_path() -> Result<PathBuf, io::Error> {
        std::env::current_exe()
    }

    /// Enable autostart by creating desktop entry
    pub fn enable() -> Result<(), io::Error> {
        let autostart_path = Self::autostart_path()?;
        let executable_path = Self::get_executable_path()?;

        let desktop_entry = format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=Lookout\n\
             Comment=Break reminder application\n\
             Exec={}\n\
             Icon=alarm-symbolic\n\
             Terminal=false\n\
             StartupNotify=false\n\
             Categories=Utility;\n\
             X-GNOME-Autostart-enabled=true\n",
            executable_path.display()
        );

        fs::write(&autostart_path, desktop_entry)?;
        log::info!("Autostart enabled: {}", autostart_path.display());

        Ok(())
    }

    /// Disable autostart by removing desktop entry
    pub fn disable() -> Result<(), io::Error> {
        let autostart_path = Self::autostart_path()?;

        if autostart_path.exists() {
            fs::remove_file(&autostart_path)?;
            log::info!("Autostart disabled: {}", autostart_path.display());
        } else {
            log::debug!("Autostart file doesn't exist, nothing to disable");
        }

        Ok(())
    }

    /// Check if autostart is currently enabled
    #[allow(dead_code)]
    pub fn is_enabled() -> bool {
        Self::autostart_path()
            .map(|path| path.exists())
            .unwrap_or(false)
    }

    /// Sync autostart state with config setting
    pub fn sync(should_enable: bool) -> Result<(), io::Error> {
        if should_enable {
            Self::enable()
        } else {
            Self::disable()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_autostart_path() {
        let path = AutostartManager::autostart_path();
        assert!(path.is_ok());

        let path = path.unwrap();
        assert!(path.to_string_lossy().contains("autostart"));
        assert!(path.to_string_lossy().ends_with("lookout.desktop"));
    }

    #[test]
    fn test_executable_path() {
        let path = AutostartManager::get_executable_path();
        assert!(path.is_ok());
    }
}
