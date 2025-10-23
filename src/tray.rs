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

/// Check for updates from tray menu
fn check_for_updates_from_tray() {
    use crate::updater;
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType};

    if !updater::can_self_update() {
        let dialog = MessageDialog::new(
            None::<&gtk4::Window>,
            DialogFlags::MODAL,
            MessageType::Warning,
            ButtonsType::Ok,
            "Self-update is disabled because Lookout was installed via a package manager.\n\nPlease use your system's package manager to update.",
        );
        dialog.set_title(Some("Self-update Disabled"));
        dialog.connect_response(move |dialog, _| {
            dialog.close();
        });
        dialog.present();
        return;
    }

    // Show checking dialog
    let checking_dialog = MessageDialog::new(
        None::<&gtk4::Window>,
        DialogFlags::MODAL,
        MessageType::Info,
        ButtonsType::None,
        "Please wait while we check for updates...",
    );
    checking_dialog.set_title(Some("Checking for Updates"));
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
                let dialog = MessageDialog::new(
                    None::<&gtk4::Window>,
                    DialogFlags::MODAL,
                    MessageType::Info,
                    ButtonsType::Ok,
                    &format!(
                        "You're already running the latest version (v{})!",
                        env!("CARGO_PKG_VERSION")
                    ),
                );
                dialog.set_title(Some("Up to Date"));
                dialog.connect_response(move |dialog, _| {
                    dialog.close();
                });
                dialog.present();
            },
            Err(e) => {
                let dialog = MessageDialog::new(
                    None::<&gtk4::Window>,
                    DialogFlags::MODAL,
                    MessageType::Error,
                    ButtonsType::Ok,
                    &e.to_string(),
                );
                dialog.set_title(Some("Update Check Failed"));
                dialog.connect_response(move |dialog, _| {
                    dialog.close();
                });
                dialog.present();
                log::error!("Update check failed: {}", e);
            },
        }
    });
}

/// Show update dialog from tray
fn show_tray_update_dialog(new_version: &str) {
    use crate::updater;
    use gtk4::{ButtonsType, DialogFlags, MessageDialog, MessageType, ResponseType};

    let message = format!(
        "A new version of Lookout is available!\n\nCurrent: v{}\nLatest: v{}\n\nWould you like to install it now?",
        env!("CARGO_PKG_VERSION"),
        new_version
    );

    let dialog = MessageDialog::new(
        None::<&gtk4::Window>,
        DialogFlags::MODAL,
        MessageType::Question,
        ButtonsType::YesNo,
        &message,
    );

    dialog.set_title(Some("Update Available"));

    let new_version = new_version.to_string();
    dialog.connect_response(move |dialog, response| {
        if response == ResponseType::Yes {
            let progress_dialog = MessageDialog::new(
                None::<&gtk4::Window>,
                DialogFlags::MODAL,
                MessageType::Info,
                ButtonsType::None,
                &format!("Downloading and installing version {}...", new_version),
            );
            progress_dialog.set_title(Some("Installing Update"));
            progress_dialog.present();

            gtk4::glib::spawn_future_local(async move {
                let result = updater::perform_update().await;

                match result {
                    Ok(_version) => {
                        progress_dialog.close();

                        let success_dialog = MessageDialog::new(
                            None::<&gtk4::Window>,
                            DialogFlags::MODAL,
                            MessageType::Info,
                            ButtonsType::Ok,
                            "Update installed successfully! Restarting Lookout...",
                        );
                        success_dialog.set_title(Some("Update Successful"));
                        success_dialog.present();

                        gtk4::glib::timeout_add_seconds_local_once(2, || {
                            if let Err(e) = updater::restart_application() {
                                log::error!("Failed to restart: {}", e);
                            }
                        });
                    },
                    Err(e) => {
                        progress_dialog.close();
                        let error_dialog = MessageDialog::new(
                            None::<&gtk4::Window>,
                            DialogFlags::MODAL,
                            MessageType::Error,
                            ButtonsType::Ok,
                            &format!("Failed to install update: {}", e),
                        );
                        error_dialog.set_title(Some("Update Failed"));
                        error_dialog.connect_response(move |dialog, _| {
                            dialog.close();
                        });
                        error_dialog.present();
                    },
                }
            });
        }
        dialog.close();
    });

    dialog.present();
}
