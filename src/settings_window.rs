//! Settings window UI using GTK4 and libadwaita

use crate::settings::Settings;
use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Orientation, SpinButton, Switch};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;

/// Settings window
pub struct SettingsWindow {
    settings: Settings,
    window: Rc<RefCell<Option<adw::ApplicationWindow>>>,
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
    #[allow(clippy::too_many_lines)]
    pub fn show(&self) {
        log::info!("Opening settings window");

        // Check if window already exists
        if let Some(window) = self.window.borrow().as_ref() {
            window.present();
            return;
        }

        // Get current config
        let config = self.settings.get();

        // Create window
        let window = adw::ApplicationWindow::builder()
            .title("Lookout Settings")
            .default_width(500)
            .default_height(400)
            .build();

        // Create header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Lookout Settings", "Configure break intervals");
        header.set_title_widget(Some(&title));

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);

        // Create preferences page
        let prefs_page = adw::PreferencesPage::new();

        // Micro break group
        let micro_group = adw::PreferencesGroup::new();
        micro_group.set_title("Micro Break");
        micro_group.set_description(Some("Short breaks to reduce eye strain"));

        // Micro break interval
        let micro_interval_row = adw::ActionRow::new();
        micro_interval_row.set_title("Interval");
        micro_interval_row.set_subtitle("Minutes between micro breaks");
        let micro_interval_spin = SpinButton::with_range(1.0, 120.0, 1.0);
        micro_interval_spin.set_value(f64::from(config.micro_break_interval_minutes));
        micro_interval_spin.set_valign(Align::Center);
        micro_interval_row.add_suffix(&micro_interval_spin);
        micro_group.add(&micro_interval_row);

        // Micro break duration
        let micro_duration_row = adw::ActionRow::new();
        micro_duration_row.set_title("Duration");
        micro_duration_row.set_subtitle("Seconds for each micro break");
        let micro_duration_spin = SpinButton::with_range(5.0, 300.0, 5.0);
        micro_duration_spin.set_value(f64::from(config.micro_break_duration_seconds));
        micro_duration_spin.set_valign(Align::Center);
        micro_duration_row.add_suffix(&micro_duration_spin);
        micro_group.add(&micro_duration_row);

        prefs_page.add(&micro_group);

        // Long break group
        let long_group = adw::PreferencesGroup::new();
        long_group.set_title("Long Break");
        long_group.set_description(Some("Extended breaks for rest and movement"));

        // Long break interval
        let long_interval_row = adw::ActionRow::new();
        long_interval_row.set_title("Interval");
        long_interval_row.set_subtitle("Minutes between long breaks");
        let long_interval_spin = SpinButton::with_range(15.0, 480.0, 5.0);
        long_interval_spin.set_value(f64::from(config.long_break_interval_minutes));
        long_interval_spin.set_valign(Align::Center);
        long_interval_row.add_suffix(&long_interval_spin);
        long_group.add(&long_interval_row);

        // Long break duration
        let long_duration_row = adw::ActionRow::new();
        long_duration_row.set_title("Duration");
        long_duration_row.set_subtitle("Minutes for each long break");
        let long_duration_spin = SpinButton::with_range(1.0, 60.0, 1.0);
        long_duration_spin.set_value(f64::from(config.long_break_duration_minutes));
        long_duration_spin.set_valign(Align::Center);
        long_duration_row.add_suffix(&long_duration_spin);
        long_group.add(&long_duration_row);

        prefs_page.add(&long_group);

        // General settings group
        let general_group = adw::PreferencesGroup::new();
        general_group.set_title("General");

        // Enable/disable breaks
        let enabled_row = adw::ActionRow::new();
        enabled_row.set_title("Enable Breaks");
        enabled_row.set_subtitle("Turn break reminders on or off");
        let enabled_switch = Switch::new();
        enabled_switch.set_active(config.enabled);
        enabled_switch.set_valign(Align::Center);
        enabled_row.add_suffix(&enabled_switch);
        enabled_row.set_activatable_widget(Some(&enabled_switch));
        general_group.add(&enabled_row);

        // Auto-start
        let autostart_row = adw::ActionRow::new();
        autostart_row.set_title("Auto-start");
        autostart_row.set_subtitle("Start automatically on system boot");
        let autostart_switch = Switch::new();
        autostart_switch.set_active(config.auto_start);
        autostart_switch.set_valign(Align::Center);
        autostart_row.add_suffix(&autostart_switch);
        autostart_row.set_activatable_widget(Some(&autostart_switch));
        general_group.add(&autostart_row);

        prefs_page.add(&general_group);

        // Add scrolled window for preferences
        let scrolled = gtk4::ScrolledWindow::new();
        scrolled.set_child(Some(&prefs_page));
        scrolled.set_vexpand(true);
        main_box.append(&scrolled);

        // Save button
        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_margin_top(12);
        button_box.set_margin_bottom(12);
        button_box.set_margin_start(12);
        button_box.set_margin_end(12);
        button_box.set_halign(Align::End);

        let save_button = Button::with_label("Save");
        save_button.add_css_class("suggested-action");

        let settings_clone = self.settings.clone();
        save_button.connect_clicked(move |_| {
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let new_config = crate::config::Config {
                micro_break_interval_minutes: micro_interval_spin.value() as u32,
                micro_break_duration_seconds: micro_duration_spin.value() as u32,
                long_break_interval_minutes: long_interval_spin.value() as u32,
                long_break_duration_minutes: long_duration_spin.value() as u32,
                auto_start: autostart_switch.is_active(),
                enabled: enabled_switch.is_active(),
            };

            if let Err(e) = settings_clone.update(|config| *config = new_config) {
                log::error!("Failed to save settings: {e}");
            } else {
                log::info!("Settings saved successfully");
            }
        });

        button_box.append(&save_button);
        main_box.append(&button_box);

        window.set_content(Some(&main_box));

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

    /// Hide the settings window
    #[allow(dead_code)]
    pub fn hide(&self) {
        log::info!("Closing settings window");
        if let Some(window) = self.window.borrow().as_ref() {
            window.close();
        }
    }
}
