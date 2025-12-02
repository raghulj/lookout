//! Settings window UI using GTK4 and libadwaita PreferencesWindow

use crate::autostart::AutostartManager;
use crate::break_window::BreakWindow;
use crate::settings::Settings;
use crate::timer::BreakType;
use crate::updater;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, ColorButton, Label, Orientation, SpinButton, Spinner, Switch,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Settings window with sidebar navigation (handled automatically by PreferencesWindow)
pub struct SettingsWindow {
    settings: Settings,
    window: Rc<RefCell<Option<adw::PreferencesWindow>>>,
}

impl SettingsWindow {
    /// Create a new settings window
    pub fn new(settings: Settings) -> Self {
        Self {
            settings,
            window: Rc::new(RefCell::new(None)),
        }
    }

    /// Build and show the settings window
    pub fn show(&self) {
        self.show_page(None);
    }

    /// Show the settings window with the About page selected
    pub fn show_about(&self) {
        self.show_page(Some("about"));
    }

    /// Build and show the settings window, optionally navigating to a specific page
    #[allow(clippy::too_many_lines)]
    fn show_page(&self, page_name: Option<&str>) {
        log::info!("Opening settings window");

        // Check if window already exists
        if let Some(window) = self.window.borrow().as_ref() {
            if let Some(name) = page_name {
                window.set_visible_page_name(name);
            }
            window.present();
            return;
        }

        // Get current config
        let config = self.settings.get();

        // Create PreferencesWindow - this handles navigation automatically
        let window = adw::PreferencesWindow::builder()
            .title("Lookout Settings")
            .default_width(700)
            .default_height(550)
            .search_enabled(true)
            .build();

        // Build and add pages
        let general_page = self.build_general_page(&config);
        let timers_page = self.build_timers_page(&config);
        let messages_page = Self::build_messages_page(&config);
        let appearance_page = self.build_appearance_page(&config);
        let about_page = Self::build_about_page();

        window.add(&general_page);
        window.add(&timers_page);
        window.add(&messages_page);
        window.add(&appearance_page);
        window.add(&about_page);

        // Navigate to requested page if specified
        if let Some(name) = page_name {
            window.set_visible_page_name(name);
        }

        // Store window reference
        *self.window.borrow_mut() = Some(window.clone());

        // Clean up reference when window is closed
        let window_ref = Rc::clone(&self.window);
        window.connect_close_request(move |_| {
            *window_ref.borrow_mut() = None;
            gtk4::glib::Propagation::Proceed
        });

        window.present();
    }

    /// Build General settings page
    fn build_general_page(&self, config: &crate::config::Config) -> adw::PreferencesPage {
        let page = adw::PreferencesPage::builder()
            .title("General")
            .icon_name("preferences-system-symbolic")
            .name("general")
            .build();

        // General Settings group
        let general_group = adw::PreferencesGroup::builder()
            .title("General Settings")
            .build();

        // Enable/disable breaks
        let enabled_row = adw::ActionRow::builder()
            .title("Enable Break Reminders")
            .subtitle("Turn break notifications on or off")
            .build();
        let enabled_switch = Switch::new();
        enabled_switch.set_active(config.enabled);
        enabled_switch.set_valign(Align::Center);
        enabled_row.add_suffix(&enabled_switch);
        enabled_row.set_activatable_widget(Some(&enabled_switch));
        general_group.add(&enabled_row);

        // Auto-start
        let autostart_row = adw::ActionRow::builder()
            .title("Start on Login")
            .subtitle("Automatically start Lookout when you log in")
            .build();
        let autostart_switch = Switch::new();
        autostart_switch.set_active(config.auto_start);
        autostart_switch.set_valign(Align::Center);
        autostart_row.add_suffix(&autostart_switch);
        autostart_row.set_activatable_widget(Some(&autostart_switch));
        general_group.add(&autostart_row);

        page.add(&general_group);

        // Updates group
        let updates_group = adw::PreferencesGroup::builder().title("Updates").build();

        // Auto-update check setting
        let auto_update_row = adw::ActionRow::builder()
            .title("Check for Updates Automatically")
            .subtitle("Check for new versions when Lookout starts")
            .build();
        let auto_update_switch = Switch::new();
        auto_update_switch.set_active(config.auto_update_check);
        auto_update_switch.set_valign(Align::Center);
        auto_update_row.add_suffix(&auto_update_switch);
        auto_update_row.set_activatable_widget(Some(&auto_update_switch));
        updates_group.add(&auto_update_row);

        // Check for updates button
        let check_updates_row = adw::ActionRow::builder()
            .title("Check for Updates")
            .subtitle(&format!("Current version: {}", env!("CARGO_PKG_VERSION")))
            .build();

        let check_button_box = GtkBox::new(Orientation::Horizontal, 6);
        check_button_box.set_valign(Align::Center);

        let check_button = Button::with_label("Check Now");
        let spinner = Spinner::new();
        spinner.set_visible(false);
        let status_label = Label::new(None);
        status_label.set_visible(false);

        check_button_box.append(&check_button);
        check_button_box.append(&spinner);
        check_button_box.append(&status_label);

        let spinner_clone = spinner.clone();
        let status_label_clone = status_label.clone();
        let check_button_clone = check_button.clone();

        check_button.connect_clicked(move |_| {
            let spinner = spinner_clone.clone();
            let status_label = status_label_clone.clone();
            let check_button = check_button_clone.clone();

            // Show spinner
            spinner.set_visible(true);
            spinner.start();
            status_label.set_visible(false);
            check_button.set_sensitive(false);

            // Check for updates in background
            let spinner = spinner.clone();
            let status_label = status_label.clone();
            let check_button = check_button.clone();

            gtk4::glib::spawn_future_local(async move {
                let result = if !updater::can_self_update() {
                    Err("Self-update disabled (installed via package manager)".to_string())
                } else {
                    updater::check_for_updates()
                        .await
                        .map_err(|e| e.to_string())
                };

                spinner.stop();
                spinner.set_visible(false);
                check_button.set_sensitive(true);
                status_label.set_visible(true);

                match result {
                    Ok(Some(new_version)) => {
                        status_label.set_text(&format!("Update available: v{}", new_version));
                        status_label.add_css_class("success");

                        // Show update dialog
                        show_update_dialog(&new_version);
                    },
                    Ok(None) => {
                        status_label.set_text("You're up to date!");
                        status_label.add_css_class("success");
                    },
                    Err(e) => {
                        status_label.set_text(&e);
                        status_label.add_css_class("error");
                        log::error!("Update check failed: {}", e);
                    },
                }
            });
        });

        check_updates_row.add_suffix(&check_button_box);
        updates_group.add(&check_updates_row);

        // Warning if can't self-update
        if !updater::can_self_update() {
            let warning_row = adw::ActionRow::builder()
                .title("Self-update disabled")
                .subtitle(
                    "Installed via package manager. Use your system's package manager to update.",
                )
                .build();
            warning_row.add_css_class("warning");
            updates_group.add(&warning_row);
        }

        page.add(&updates_group);

        // Save button group
        // Idle detection group
        let idle_group = adw::PreferencesGroup::new();
        idle_group.set_title("Idle Detection");
        idle_group.set_description(Some(
            "Reset break timers when you return after being away from the computer",
        ));
        idle_group.set_margin_bottom(24);

        // Enable idle detection
        let idle_enabled_row = adw::ActionRow::new();
        idle_enabled_row.set_title("Enable Idle Detection");
        idle_enabled_row.set_subtitle(
            "Automatically reset timers if you've been away (no keyboard/mouse activity)",
        );
        let idle_enabled_switch = Switch::new();
        idle_enabled_switch.set_active(config.idle_detection_enabled);
        idle_enabled_switch.set_valign(Align::Center);
        idle_enabled_row.add_suffix(&idle_enabled_switch);
        idle_enabled_row.set_activatable_widget(Some(&idle_enabled_switch));
        idle_group.add(&idle_enabled_row);

        // Idle threshold
        let idle_threshold_row = adw::ActionRow::new();
        idle_threshold_row.set_title("Idle Threshold");
        idle_threshold_row.set_subtitle("Minutes of inactivity before considering you 'away'");
        let idle_threshold_spin = SpinButton::with_range(1.0, 30.0, 1.0);
        idle_threshold_spin.set_value(f64::from(config.idle_threshold_minutes));
        idle_threshold_spin.set_valign(Align::Center);
        idle_threshold_row.add_suffix(&idle_threshold_spin);
        idle_group.add(&idle_threshold_row);

        // Enable/disable threshold spin based on idle detection toggle
        idle_threshold_spin.set_sensitive(config.idle_detection_enabled);
        let idle_threshold_spin_clone = idle_threshold_spin.clone();
        idle_enabled_switch.connect_state_set(move |_, state| {
            idle_threshold_spin_clone.set_sensitive(state);
            gtk4::glib::Propagation::Proceed
        });

        page.add(&idle_group);

        // Save button in its own group
        let button_group = adw::PreferencesGroup::new();

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::End);
        button_box.set_margin_top(12);

        let save_button = Button::with_label("Save Settings");
        save_button.add_css_class("suggested-action");

        let settings_clone = self.settings.clone();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        save_button.connect_clicked(move |_| {
            let new_auto_start = autostart_switch.is_active();

            if let Err(e) = settings_clone.update(|cfg| {
                cfg.enabled = enabled_switch.is_active();
                cfg.auto_start = new_auto_start;
                cfg.auto_update_check = auto_update_switch.is_active();
                cfg.idle_detection_enabled = idle_enabled_switch.is_active();
                cfg.idle_threshold_minutes = idle_threshold_spin.value() as u32;
            }) {
                log::error!("Failed to save general settings: {e}");
            } else {
                log::info!("General settings saved successfully");

                // Sync autostart state with system
                if let Err(e) = AutostartManager::sync(new_auto_start) {
                    log::error!("Failed to sync autostart setting: {e}");
                } else {
                    log::info!("Autostart setting synced successfully");
                }
            }
        });

        button_box.append(&save_button);
        button_group.add(&button_box);
        page.add(&button_group);

        page
    }

    /// Build Timers settings page
    fn build_timers_page(&self, config: &crate::config::Config) -> adw::PreferencesPage {
        let page = adw::PreferencesPage::builder()
            .title("Timers")
            .icon_name("alarm-symbolic")
            .name("timers")
            .build();

        // Micro break group
        let micro_group = adw::PreferencesGroup::builder()
            .title("Micro Break")
            .description("Short breaks to reduce eye strain (20-20-20 rule)")
            .build();

        let micro_interval_row = adw::ActionRow::builder()
            .title("Interval")
            .subtitle("Minutes between micro breaks")
            .build();
        let micro_interval_spin = SpinButton::with_range(1.0, 120.0, 1.0);
        micro_interval_spin.set_value(f64::from(config.micro_break_interval_minutes));
        micro_interval_spin.set_valign(Align::Center);
        micro_interval_row.add_suffix(&micro_interval_spin);
        micro_group.add(&micro_interval_row);

        let micro_duration_row = adw::ActionRow::builder()
            .title("Duration")
            .subtitle("Seconds for each micro break")
            .build();
        let micro_duration_spin = SpinButton::with_range(5.0, 300.0, 5.0);
        micro_duration_spin.set_value(f64::from(config.micro_break_duration_seconds));
        micro_duration_spin.set_valign(Align::Center);
        micro_duration_row.add_suffix(&micro_duration_spin);
        micro_group.add(&micro_duration_row);

        page.add(&micro_group);

        // Long break group
        let long_group = adw::PreferencesGroup::builder()
            .title("Long Break")
            .description("Extended breaks for rest and movement")
            .build();

        let long_interval_row = adw::ActionRow::builder()
            .title("Interval")
            .subtitle("Minutes between long breaks")
            .build();
        let long_interval_spin = SpinButton::with_range(15.0, 480.0, 5.0);
        long_interval_spin.set_value(f64::from(config.long_break_interval_minutes));
        long_interval_spin.set_valign(Align::Center);
        long_interval_row.add_suffix(&long_interval_spin);
        long_group.add(&long_interval_row);

        let long_duration_row = adw::ActionRow::builder()
            .title("Duration")
            .subtitle("Minutes for each long break")
            .build();
        let long_duration_spin = SpinButton::with_range(1.0, 60.0, 1.0);
        long_duration_spin.set_value(f64::from(config.long_break_duration_minutes));
        long_duration_spin.set_valign(Align::Center);
        long_duration_row.add_suffix(&long_duration_spin);
        long_group.add(&long_duration_row);

        page.add(&long_group);

        // Save button
        let button_group = adw::PreferencesGroup::new();

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::End);
        button_box.set_margin_top(12);

        let save_button = Button::with_label("Save Settings");
        save_button.add_css_class("suggested-action");

        let settings_clone = self.settings.clone();
        save_button.connect_clicked(move |_| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            if let Err(e) = settings_clone.update(|cfg| {
                cfg.micro_break_interval_minutes = micro_interval_spin.value() as u32;
                cfg.micro_break_duration_seconds = micro_duration_spin.value() as u32;
                cfg.long_break_interval_minutes = long_interval_spin.value() as u32;
                cfg.long_break_duration_minutes = long_duration_spin.value() as u32;
            }) {
                log::error!("Failed to save timer settings: {e}");
            } else {
                log::info!("Timer settings saved successfully");
            }
        });

        button_box.append(&save_button);
        button_group.add(&button_box);
        page.add(&button_group);

        page
    }

    /// Build Messages management page
    fn build_messages_page(config: &crate::config::Config) -> adw::PreferencesPage {
        let page = adw::PreferencesPage::builder()
            .title("Messages")
            .icon_name("mail-message-symbolic")
            .name("messages")
            .build();

        // Info note at top
        let info_group = adw::PreferencesGroup::new();
        let info_row = adw::ActionRow::builder()
            .title("Random Message Selection")
            .subtitle("One message from each group is randomly selected every break")
            .build();
        info_group.add(&info_row);
        page.add(&info_group);

        // Headings group
        let headings_group = adw::PreferencesGroup::builder()
            .title("Break Headings")
            .description("Main messages shown at the top of break screens")
            .build();

        for (i, heading) in config.break_messages.headings.iter().enumerate() {
            let row = adw::ActionRow::builder()
                .title(&format!("{}. {}", i + 1, heading))
                .build();
            headings_group.add(&row);
        }

        page.add(&headings_group);

        // Micro break instructions
        let micro_group = adw::PreferencesGroup::builder()
            .title("Micro Break Messages")
            .description("Instructions for 20-second eye rest breaks")
            .build();

        for (i, instruction) in config
            .break_messages
            .micro_break_instructions
            .iter()
            .enumerate()
        {
            let row = adw::ActionRow::builder()
                .title(&format!("{}. {}", i + 1, instruction))
                .build();
            micro_group.add(&row);
        }

        page.add(&micro_group);

        // Long break instructions
        let long_group = adw::PreferencesGroup::builder()
            .title("Long Break Messages")
            .description("Instructions for 5-minute movement breaks")
            .build();

        for (i, instruction) in config
            .break_messages
            .long_break_instructions
            .iter()
            .enumerate()
        {
            let row = adw::ActionRow::builder()
                .title(&format!("{}. {}", i + 1, instruction))
                .build();
            long_group.add(&row);
        }

        page.add(&long_group);

        // Note about editing
        let note_group = adw::PreferencesGroup::new();
        let note_row = adw::ActionRow::builder()
            .title("Note")
            .subtitle("Message editing via UI is coming soon. You can manually edit ~/.config/lookout/config.json")
            .build();
        note_group.add(&note_row);
        page.add(&note_group);

        page
    }

    /// Build Appearance settings page
    #[allow(clippy::too_many_lines)]
    fn build_appearance_page(&self, config: &crate::config::Config) -> adw::PreferencesPage {
        let page = adw::PreferencesPage::builder()
            .title("Appearance")
            .icon_name("preferences-desktop-wallpaper-symbolic")
            .name("appearance")
            .build();

        let appearance_group = adw::PreferencesGroup::builder()
            .title("Break Window Colors")
            .description("Customize the fullscreen break overlay")
            .build();

        // Background color
        let bg_color_row = adw::ActionRow::builder()
            .title("Background Color")
            .subtitle("Click to choose a background color")
            .build();

        let bg_color_button = ColorButton::new();
        if let Some(rgba) = parse_color_string(&config.background_color) {
            bg_color_button.set_rgba(&rgba);
        }
        bg_color_button.set_valign(Align::Center);
        bg_color_row.add_suffix(&bg_color_button);
        appearance_group.add(&bg_color_row);

        // Text color
        let text_color_row = adw::ActionRow::builder()
            .title("Text Color")
            .subtitle("Click to choose a text color")
            .build();

        let text_color_button = ColorButton::new();
        if let Some(rgba) = parse_color_string(&config.text_color) {
            text_color_button.set_rgba(&rgba);
        }
        text_color_button.set_valign(Align::Center);
        text_color_row.add_suffix(&text_color_button);
        appearance_group.add(&text_color_row);

        page.add(&appearance_group);

        // Presets
        let presets_group = adw::PreferencesGroup::builder()
            .title("Quick Presets")
            .description("Apply preset color combinations")
            .build();

        let presets = vec![
            (
                "Default Dark",
                "rgba(0, 0, 0, 0.95)",
                "rgba(255, 255, 255, 1.0)",
                "Black background with white text",
            ),
            (
                "Pure Black",
                "rgba(0, 0, 0, 1.0)",
                "rgba(255, 255, 255, 1.0)",
                "Solid black with white text",
            ),
            (
                "Navy Blue",
                "rgba(15, 23, 42, 0.95)",
                "rgba(226, 232, 240, 1.0)",
                "Dark blue with light gray text",
            ),
            (
                "Deep Purple",
                "rgba(30, 20, 60, 0.95)",
                "rgba(243, 232, 255, 1.0)",
                "Purple background with lavender text",
            ),
        ];

        for (name, bg_color, text_color, description) in presets {
            let preset_row = adw::ActionRow::builder()
                .title(name)
                .subtitle(description)
                .build();

            let apply_button = Button::with_label("Apply");
            apply_button.set_valign(Align::Center);

            let bg_button_clone = bg_color_button.clone();
            let text_button_clone = text_color_button.clone();
            let bg_color_str = bg_color.to_string();
            let text_color_str = text_color.to_string();

            apply_button.connect_clicked(move |_| {
                if let Some(rgba) = parse_color_string(&bg_color_str) {
                    bg_button_clone.set_rgba(&rgba);
                }
                if let Some(rgba) = parse_color_string(&text_color_str) {
                    text_button_clone.set_rgba(&rgba);
                }
            });

            preset_row.add_suffix(&apply_button);
            presets_group.add(&preset_row);
        }

        page.add(&presets_group);

        // Buttons
        let button_group = adw::PreferencesGroup::new();

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::End);
        button_box.set_margin_top(12);

        // Preview button
        let preview_button = Button::with_label("Preview");

        let settings_for_preview = self.settings.clone();
        let bg_button_for_preview = bg_color_button.clone();
        let text_button_for_preview = text_color_button.clone();

        preview_button.connect_clicked(move |_| {
            let bg_rgba = bg_button_for_preview.rgba();
            let text_rgba = text_button_for_preview.rgba();

            let bg_color_str = rgba_to_string(&bg_rgba);
            let text_color_str = rgba_to_string(&text_rgba);

            // Create temporary config with selected colors
            let mut temp_config = settings_for_preview.get();
            temp_config.background_color = bg_color_str;
            temp_config.text_color = text_color_str;

            // Show preview break window
            let break_window = BreakWindow::new(temp_config);
            break_window.show(BreakType::Micro, Duration::from_secs(10));

            log::info!("Showing preview with selected colors");
        });

        button_box.append(&preview_button);

        // Save button
        let save_button = Button::with_label("Save Settings");
        save_button.add_css_class("suggested-action");

        let settings_clone = self.settings.clone();
        save_button.connect_clicked(move |_| {
            let bg_rgba = bg_color_button.rgba();
            let text_rgba = text_color_button.rgba();

            if let Err(e) = settings_clone.update(|cfg| {
                cfg.background_color = rgba_to_string(&bg_rgba);
                cfg.text_color = rgba_to_string(&text_rgba);
            }) {
                log::error!("Failed to save appearance settings: {e}");
            } else {
                log::info!("Appearance settings saved successfully");
            }
        });

        button_box.append(&save_button);
        button_group.add(&button_box);
        page.add(&button_group);

        page
    }

    /// Build About page
    fn build_about_page() -> adw::PreferencesPage {
        let page = adw::PreferencesPage::builder()
            .title("About")
            .icon_name("help-about-symbolic")
            .name("about")
            .build();

        let about_group = adw::PreferencesGroup::builder()
            .title("About Lookout")
            .description("Break reminder application for Linux")
            .build();

        let version_row = adw::ActionRow::builder()
            .title("Version")
            .subtitle(env!("CARGO_PKG_VERSION"))
            .build();
        about_group.add(&version_row);

        let desc_row = adw::ActionRow::builder()
            .title("Description")
            .subtitle(
                "A lightweight break reminder app to reduce eye strain and promote healthy breaks",
            )
            .build();
        about_group.add(&desc_row);

        page.add(&about_group);

        let dev_group = adw::PreferencesGroup::builder()
            .title("Development")
            .build();

        let ai_row = adw::ActionRow::builder()
            .title("Built with AI")
            .subtitle("Developed and maintained entirely using Claude Code (AI)")
            .build();
        dev_group.add(&ai_row);

        page.add(&dev_group);

        let links_group = adw::PreferencesGroup::builder()
            .title("Links and Resources")
            .build();

        let github_row = adw::ActionRow::builder()
            .title("GitHub Repository")
            .subtitle("https://github.com/raghulj/lookout")
            .build();
        links_group.add(&github_row);

        let license_row = adw::ActionRow::builder()
            .title("License")
            .subtitle("MIT License - Free and open source")
            .build();
        links_group.add(&license_row);

        page.add(&links_group);

        page
    }

    /// Hide the settings window
    #[allow(dead_code)]
    pub fn hide(&self) {
        log::info!("Closing settings window");
        if let Some(window) = self.window.borrow().as_ref() {
            window.close();
        }
    }
}

/// Parse CSS rgba color string to GTK RGBA
fn parse_color_string(color_str: &str) -> Option<gtk4::gdk::RGBA> {
    // Try to parse rgba(r, g, b, a) format
    if let Some(inner) = color_str
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 4 {
            if let (Ok(r), Ok(g), Ok(b), Ok(a)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
                parts[3].parse::<f32>(),
            ) {
                let mut rgba = gtk4::gdk::RGBA::new(0.0, 0.0, 0.0, 1.0);
                rgba.set_red(f32::from(r) / 255.0);
                rgba.set_green(f32::from(g) / 255.0);
                rgba.set_blue(f32::from(b) / 255.0);
                rgba.set_alpha(a);
                return Some(rgba);
            }
        }
    }
    None
}

/// Convert GTK RGBA to CSS rgba string
fn rgba_to_string(rgba: &gtk4::gdk::RGBA) -> String {
    let r = (rgba.red() * 255.0).round() as u8;
    let g = (rgba.green() * 255.0).round() as u8;
    let b = (rgba.blue() * 255.0).round() as u8;
    let a = rgba.alpha();
    format!("rgba({}, {}, {}, {})", r, g, b, a)
}

/// Show update available dialog with option to install
fn show_update_dialog(new_version: &str) {
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
            perform_update_with_progress(&new_version);
        }
    });

    dialog.present();
}

/// Perform update with progress indication
fn perform_update_with_progress(new_version: &str) {
    let dialog = adw::MessageDialog::new(
        None::<&gtk4::Window>,
        Some("Installing Update"),
        Some(&format!(
            "Downloading and installing version {}...\n\nThis may take a moment.",
            new_version
        )),
    );

    // No buttons - this is a progress dialog
    dialog.present();

    let dialog_clone = dialog.clone();

    gtk4::glib::spawn_future_local(async move {
        let result = updater::perform_update().await;

        match result {
            Ok(version) => {
                dialog_clone.set_body("Update successful! Restarting...");

                // Wait a moment for user to see the message
                gtk4::glib::timeout_add_seconds_local_once(2, move || {
                    log::info!("Update completed to version {}. Restarting...", version);

                    // Restart the application
                    if let Err(e) = updater::restart_application() {
                        log::error!("Failed to restart application: {}", e);
                        show_error_dialog(
                            "Failed to restart application. Please restart manually.",
                        );
                    }
                });
            },
            Err(e) => {
                dialog_clone.close();
                show_error_dialog(&format!("Update failed: {}", e));
                log::error!("Update failed: {}", e);
            },
        }
    });
}

/// Show error dialog
fn show_error_dialog(message: &str) {
    let dialog =
        adw::MessageDialog::new(None::<&gtk4::Window>, Some("Update Error"), Some(message));

    dialog.add_response("ok", "OK");
    dialog.set_default_response(Some("ok"));

    dialog.present();
}
