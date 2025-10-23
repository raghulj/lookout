//! System tray integration using ksni

use crate::settings::Settings;
use crate::timer::BreakType;
use gtk4::prelude::*;
use ksni::{menu, Tray, TrayService as KsniTrayService};
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;

/// Tray icon status
struct LookoutTray {
    settings: Settings,
    tooltip: Arc<RwLock<String>>,
    #[allow(dead_code)] // Used in menu activate closures
    test_break_tx: mpsc::UnboundedSender<BreakType>,
}

impl Tray for LookoutTray {
    fn icon_name(&self) -> String {
        // Use a standard icon name from freedesktop.org icon naming spec
        // In production, this would be a custom icon
        "appointment-soon".to_string()
    }

    fn title(&self) -> String {
        "Lookout".to_string()
    }

    fn tool_tip(&self) -> ksni::ToolTip {
        let tooltip_text = self
            .tooltip
            .read()
            .map_or_else(|_| "Lookout".to_string(), |t| t.clone());

        ksni::ToolTip {
            title: "Lookout".to_string(),
            description: tooltip_text,
            icon_name: String::new(),
            icon_pixmap: vec![],
        }
    }

    fn menu(&self) -> Vec<menu::MenuItem<Self>> {
        use menu::{MenuItem, StandardItem};

        let mut items = vec![];

        // Add test break items only in debug builds
        #[cfg(debug_assertions)]
        {
            items.push(
                StandardItem {
                    label: "Test Micro Break (20s)".to_string(),
                    icon_name: "media-playback-start".to_string(),
                    activate: Box::new(|this: &mut Self| {
                        log::info!("Testing micro break from tray");
                        let _ = this.test_break_tx.send(BreakType::Micro);
                    }),
                    ..Default::default()
                }
                .into(),
            );

            items.push(
                StandardItem {
                    label: "Test Long Break (5m)".to_string(),
                    icon_name: "media-playback-start".to_string(),
                    activate: Box::new(|this: &mut Self| {
                        log::info!("Testing long break from tray");
                        let _ = this.test_break_tx.send(BreakType::Long);
                    }),
                    ..Default::default()
                }
                .into(),
            );

            items.push(MenuItem::Separator);
        }

        // Settings
        items.push(
            StandardItem {
                label: "Settings".to_string(),
                icon_name: "preferences-system".to_string(),
                activate: Box::new(|this: &mut Self| {
                    log::info!("Opening settings from tray");
                    let settings = this.settings.clone();
                    gtk4::glib::MainContext::default().invoke(move || {
                        let settings_window = crate::settings_window::SettingsWindow::new(settings);
                        settings_window.show();
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // About
        items.push(
            StandardItem {
                label: "About".to_string(),
                icon_name: "help-about".to_string(),
                activate: Box::new(|_this: &mut Self| {
                    log::info!("About clicked");
                    gtk4::glib::MainContext::default().invoke(|| {
                        show_about_dialog();
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // Quit
        items.push(
            StandardItem {
                label: "Quit".to_string(),
                icon_name: "application-exit".to_string(),
                activate: Box::new(|_this: &mut Self| {
                    log::info!("Quitting application from tray");
                    std::process::exit(0);
                }),
                ..Default::default()
            }
            .into(),
        );

        items
    }
}

/// System tray service
pub struct TrayService {
    settings: Settings,
    tooltip: Arc<RwLock<String>>,
    test_break_tx: mpsc::UnboundedSender<BreakType>,
    handle: Option<ksni::Handle<LookoutTray>>,
}

impl TrayService {
    /// Create a new tray service
    pub fn new(settings: Settings) -> (Self, mpsc::UnboundedReceiver<BreakType>) {
        let (test_break_tx, test_break_rx) = mpsc::unbounded_channel();

        let service = Self {
            settings,
            tooltip: Arc::new(RwLock::new("Next break in...".to_string())),
            test_break_tx,
            handle: None,
        };

        (service, test_break_rx)
    }

    /// Initialize and show the tray icon
    pub fn show(&mut self) {
        log::info!("Initializing system tray");

        let tray = LookoutTray {
            settings: self.settings.clone(),
            tooltip: Arc::clone(&self.tooltip),
            test_break_tx: self.test_break_tx.clone(),
        };

        // Spawn the tray service and keep the handle alive
        let service = KsniTrayService::new(tray);
        let handle = service.handle();
        service.spawn();

        self.handle = Some(handle);

        log::info!("System tray initialized successfully");
    }

    /// Update tray tooltip with next break time
    pub fn update_tooltip(&self, text: &str) {
        if let Ok(mut tooltip) = self.tooltip.write() {
            *tooltip = text.to_string();
            log::debug!("Updated tray tooltip: {text}");

            // Notify the tray to refresh its tooltip
            if let Some(handle) = &self.handle {
                handle.update(|_tray| {
                    // The update callback triggers a refresh
                });
            }
        }
    }

    /// Hide the tray icon
    #[allow(dead_code, clippy::unused_self)]
    pub fn hide(&self) {
        log::info!("Hiding system tray");
        // ksni doesn't provide an explicit hide method
        // The tray will be removed when the service is dropped
    }
}

/// Show the About dialog
fn show_about_dialog() {
    let dialog = gtk4::AboutDialog::builder()
        .program_name("Lookout")
        .version("0.1.0")
        .license_type(gtk4::License::MitX11)
        .website("https://github.com/raghulj/lookout")
        .copyright("© 2024")
        .comments("A lightweight break reminder app for Linux that helps reduce eye strain and digital fatigue.\n\n🤖 This application was developed and is maintained entirely using Claude Code (AI).")
        .logo_icon_name("appointment-soon")
        .modal(true)
        .build();

    // Add developers
    dialog.set_authors(&["Developed by Claude Code (AI)"]);

    // Add designers
    dialog.set_artists(&["Claude Code (AI)"]);

    dialog.present();
}
