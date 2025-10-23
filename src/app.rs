//! Main application setup and GTK initialization

use crate::{
    autostart::AutostartManager, break_window::BreakWindow, settings::Settings, timer::TimerEngine,
    tray::TrayService, updater,
};
use gtk4::prelude::*;
use gtk4::{glib, Application};
use std::error::Error;
use std::sync::Arc;

const APP_ID: &str = "com.github.lookout";

pub struct LookoutApp {
    settings: Settings,
}

impl LookoutApp {
    /// Create a new application instance
    pub fn new() -> Self {
        Self {
            settings: Settings::new(),
        }
    }

    /// Run the application
    pub fn run(self) -> Result<(), Box<dyn Error>> {
        log::info!("Initializing GTK4 application");

        // Initialize GTK4 application
        // GTK4 Application automatically ensures single instance through D-Bus
        // If another instance is running, this one will exit automatically
        let app = Application::builder().application_id(APP_ID).build();

        let settings = self.settings;

        // Register the application early to check for existing instance
        // This will fail if another instance with the same ID is already running
        app.register(None::<&gtk4::gio::Cancellable>)?;

        if app.is_remote() {
            log::info!("Another instance of Lookout is already running.");
            log::info!("Only one instance can run at a time. Exiting.");
            return Ok(());
        }

        log::info!("No other instance detected. Starting application...");

        // Connect to activate signal
        app.connect_activate(move |app| {
            log::info!("Application activated");

            // Initialize timer engine
            let config = settings.get();

            // Sync autostart state on startup
            if let Err(e) = AutostartManager::sync(config.auto_start) {
                log::warn!("Failed to sync autostart setting on startup: {e}");
            }

            // Check for updates on startup if enabled
            if config.auto_update_check && updater::can_self_update() {
                log::info!("Checking for updates on startup");
                tokio::spawn(async {
                    match updater::check_for_updates().await {
                        Ok(Some(new_version)) => {
                            log::info!("Update available: v{}", new_version);
                            // Show notification via GTK on main thread using invoke
                            glib::MainContext::default().invoke(move || {
                                glib::spawn_future_local(async move {
                                    show_update_notification(&new_version);
                                });
                            });
                        }
                        Ok(None) => {
                            log::info!("Already on latest version");
                        }
                        Err(e) => {
                            log::warn!("Update check failed: {}", e);
                        }
                    }
                });
            }

            let timer = Arc::new(TimerEngine::new(config.clone()));

            // Subscribe to config updates and update timer when settings change
            let mut config_receiver = settings.subscribe();
            let timer_for_config = Arc::clone(&timer);
            tokio::spawn(async move {
                while let Ok(new_config) = config_receiver.recv().await {
                    log::info!("Config updated, resetting timer");
                    timer_for_config.update_config(new_config).await;
                }
            });

            // Subscribe to break events
            let mut event_receiver = timer.subscribe();

            // Handle break events on GTK main thread
            // Clone settings to use in the event handler
            let settings_clone = settings.clone();
            glib::spawn_future_local(async move {
                while let Ok(event) = event_receiver.recv().await {
                    log::debug!("Received break event: {event:?}");

                    match event {
                        crate::timer::BreakEvent::BreakStarted(break_type, duration) => {
                            log::info!("Break started: {break_type:?} for {duration:?}");
                            // Get fresh config each time to pick up any setting changes
                            let current_config = settings_clone.get();
                            let break_window = BreakWindow::new(current_config);
                            break_window.show(break_type, duration);
                        }
                        crate::timer::BreakEvent::BreakEnded(break_type) => {
                            log::info!("Break ended: {break_type:?}");
                            // Window closes automatically
                        }
                    }
                }
            });

            // Start timer in background only if enabled
            if config.enabled {
                log::info!("Break reminders are enabled - starting timer");
                let timer_clone = Arc::clone(&timer);
                tokio::spawn(async move {
                    timer_clone.start().await;
                });
            } else {
                log::info!("Break reminders are disabled - timer not started");
            }

            // Initialize system tray
            let (mut tray, mut test_break_rx) = TrayService::new(settings.clone());
            tray.show();

            // Handle test break requests from tray
            let timer_clone = Arc::clone(&timer);
            tokio::spawn(async move {
                while let Some(break_type) = test_break_rx.recv().await {
                    log::info!("Triggering test break: {break_type:?}");
                    timer_clone.trigger_break(break_type).await;
                }
            });

            // Update tray tooltip periodically with timer information
            let tray_clone = std::sync::Arc::new(tray);
            let timer_clone = Arc::clone(&timer);
            let tray_update = Arc::clone(&tray_clone);

            tokio::spawn(async move {
                let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
                loop {
                    interval.tick().await;

                    // Get time until next breaks
                    let micro_time = timer_clone.time_until_micro_break().await;
                    let long_time = timer_clone.time_until_long_break().await;

                    // Format tooltip text (round up to next minute)
                    let micro_mins = (micro_time.as_secs() + 59) / 60; // Round up
                    let long_mins = (long_time.as_secs() + 59) / 60; // Round up

                    let tooltip = format!(
                        "Lookout - Break Reminder\n\nNext short reminder: {micro_mins}m\nNext long reminder: {long_mins}m"
                    );

                    tray_update.update_tooltip(&tooltip);
                }
            });

            // Keep tray alive by preventing it from being dropped
            std::mem::forget(tray_clone);

            // Keep application running in background - leak the hold guard so it never drops
            let hold = app.hold();
            Box::leak(Box::new(hold));

            log::info!("Application initialized successfully - running in background");
        });

        // Run the GTK application
        log::info!("Starting GTK main loop");
        app.run();

        Ok(())
    }
}

impl Default for LookoutApp {
    fn default() -> Self {
        Self::new()
    }
}

/// Show update notification when new version is available
fn show_update_notification(new_version: &str) {
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};

    let dialog = MessageDialog::new(
        None::<&gtk4::Window>,
        DialogFlags::MODAL,
        MessageType::Info,
        ButtonsType::Ok,
        &format!(
            "A new version of Lookout is available!\n\nCurrent: v{}\nLatest: v{}\n\nYou can install it from the Settings window.",
            env!("CARGO_PKG_VERSION"),
            new_version
        ),
    );

    dialog.set_title(Some("Update Available"));
    dialog.connect_response(move |dialog, _| {
        dialog.close();
    });

    dialog.present();
}
