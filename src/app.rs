//! Main application setup and GTK initialization

use crate::{
    autostart::AutostartManager, break_window::BreakWindow, settings::Settings, timer::TimerEngine,
    tray::TrayService,
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

            // Load libadwaita stylesheet
            let provider = gtk4::CssProvider::new();
            provider.load_from_path("resources/style.css");
            gtk4::style_context_add_provider_for_display(
                &gtk4::gdk::Display::default().expect("Could not connect to display"),
                &provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );

            // Initialize timer engine
            let config = settings.get();

            // Sync autostart state on startup
            if let Err(e) = AutostartManager::sync(config.auto_start) {
                log::warn!("Failed to sync autostart setting on startup: {e}");
            }

            let timer = Arc::new(TimerEngine::new(config.clone()));

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

                    // Format tooltip text
                    let micro_mins = micro_time.as_secs() / 60;
                    let micro_secs = micro_time.as_secs() % 60;
                    let long_mins = long_time.as_secs() / 60;
                    let long_secs = long_time.as_secs() % 60;

                    let tooltip = format!(
                        "Lookout - Break Reminder\n\nNext short reminder: {micro_mins}m {micro_secs}s\nNext long reminder: {long_mins}m {long_secs}s"
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
