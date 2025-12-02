//! System tray integration using ksni

use crate::settings::Settings;
use crate::timer::BreakType;
use gtk4::prelude::*;
use ksni::{menu, Tray, TrayService as KsniTrayService};
use libadwaita as adw;
use libadwaita::prelude::*;
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

        // Check for Updates
        items.push(
            StandardItem {
                label: "Check for Updates".to_string(),
                icon_name: "system-software-update".to_string(),
                activate: Box::new(|_this: &mut Self| {
                    log::info!("Checking for updates from tray");
                    gtk4::glib::MainContext::default().invoke(|| {
                        check_for_updates_from_tray();
                    });
                }),
                ..Default::default()
            }
            .into(),
        );

        items.push(MenuItem::Separator);

        // About - opens Settings window on About tab
        items.push(
            StandardItem {
                label: "About".to_string(),
                icon_name: "help-about".to_string(),
                activate: Box::new(|this: &mut Self| {
                    log::info!("Opening About from tray");
                    let settings = this.settings.clone();
                    gtk4::glib::MainContext::default().invoke(move || {
                        let settings_window = crate::settings_window::SettingsWindow::new(settings);
                        settings_window.show_about();
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

/// Check for updates from tray menu
fn check_for_updates_from_tray() {
    use crate::updater;

    if !updater::can_self_update() {
        let dialog = adw::MessageDialog::new(
            None::<&gtk4::Window>,
            Some("Self-update Disabled"),
            Some("Self-update is disabled because Lookout was installed via a package manager.\n\nPlease use your system's package manager to update."),
        );
        dialog.add_response("ok", "OK");
        dialog.set_default_response(Some("ok"));
        dialog.present();
        return;
    }

    // Show checking dialog
    let checking_dialog = adw::MessageDialog::new(
        None::<&gtk4::Window>,
        Some("Checking for Updates"),
        Some("Please wait while we check for updates..."),
    );
    checking_dialog.present();

    // Check for updates in background
    gtk4::glib::spawn_future_local(async move {
        let result = updater::check_for_updates().await;

        checking_dialog.close();

        match result {
            Ok(Some(new_version)) => {
                show_tray_update_dialog(&new_version);
            },
            Ok(None) => {
                let dialog = adw::MessageDialog::new(
                    None::<&gtk4::Window>,
                    Some("Up to Date"),
                    Some(&format!(
                        "You're already running the latest version (v{})!",
                        env!("CARGO_PKG_VERSION")
                    )),
                );
                dialog.add_response("ok", "OK");
                dialog.set_default_response(Some("ok"));
                dialog.present();
            },
            Err(e) => {
                let dialog = adw::MessageDialog::new(
                    None::<&gtk4::Window>,
                    Some("Update Check Failed"),
                    Some(&e.to_string()),
                );
                dialog.add_response("ok", "OK");
                dialog.set_default_response(Some("ok"));
                dialog.present();
                log::error!("Update check failed: {}", e);
            },
        }
    });
}

/// Show update dialog from tray
fn show_tray_update_dialog(new_version: &str) {
    use crate::updater;

    let dialog = adw::MessageDialog::new(
        None::<&gtk4::Window>,
        Some("Update Available"),
        Some(&format!(
            "A new version of Lookout is available!\n\nCurrent: v{}\nLatest: v{}\n\nWould you like to install it now?",
            env!("CARGO_PKG_VERSION"),
            new_version
        )),
    );

    dialog.add_response("no", "No");
    dialog.add_response("yes", "Yes");
    dialog.set_response_appearance("yes", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("yes"));

    let new_version = new_version.to_string();
    dialog.connect_response(None, move |dlg, response| {
        dlg.close();
        if response == "yes" {
            let progress_dialog = adw::MessageDialog::new(
                None::<&gtk4::Window>,
                Some("Installing Update"),
                Some(&format!(
                    "Downloading and installing version {}...",
                    new_version
                )),
            );
            progress_dialog.present();

            let progress_dialog_clone = progress_dialog.clone();
            let new_version_clone = new_version.clone();

            gtk4::glib::spawn_future_local(async move {
                let result = updater::perform_update().await;

                match result {
                    Ok(_version) => {
                        progress_dialog_clone.close();

                        let success_dialog = adw::MessageDialog::new(
                            None::<&gtk4::Window>,
                            Some("Update Successful"),
                            Some("Update installed successfully! Restarting Lookout..."),
                        );
                        success_dialog.add_response("ok", "OK");
                        success_dialog.present();

                        gtk4::glib::timeout_add_seconds_local_once(2, || {
                            if let Err(e) = updater::restart_application() {
                                log::error!("Failed to restart: {}", e);
                            }
                        });
                    },
                    Err(e) => {
                        progress_dialog_clone.close();
                        let error_dialog = adw::MessageDialog::new(
                            None::<&gtk4::Window>,
                            Some("Update Failed"),
                            Some(&format!("Failed to install update: {}", e)),
                        );
                        error_dialog.add_response("ok", "OK");
                        error_dialog.set_default_response(Some("ok"));
                        error_dialog.present();
                        log::error!("Update failed for version {}: {}", new_version_clone, e);
                    },
                }
            });
        }
    });

    dialog.present();
}
