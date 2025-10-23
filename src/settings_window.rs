//! Settings window UI using GTK4 and libadwaita with sidebar navigation

use crate::autostart::AutostartManager;
use crate::break_window::BreakWindow;
use crate::settings::Settings;
use crate::timer::BreakType;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, ColorButton, ListBox, Orientation, ScrolledWindow, SpinButton, Stack,
    Switch,
};
use libadwaita as adw;
use libadwaita::prelude::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Settings window with sidebar navigation
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
            .default_width(800)
            .default_height(600)
            .build();

        // Create header bar
        let header = adw::HeaderBar::new();
        let title = adw::WindowTitle::new("Settings", "");
        header.set_title_widget(Some(&title));

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.append(&header);

        // Create horizontal split view (sidebar + content)
        let content_box = GtkBox::new(Orientation::Horizontal, 0);
        content_box.set_vexpand(true);

        // Create sidebar
        let sidebar = Self::create_sidebar();
        sidebar.set_width_request(200);
        sidebar.add_css_class("navigation-sidebar");

        // Create stack for different pages
        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::SlideLeftRight);

        // Build pages
        let general_page = self.build_general_page(&config);
        let timers_page = self.build_timers_page(&config);
        let messages_page = Self::build_messages_page(&config);
        let appearance_page = self.build_appearance_page(&config);
        let about_page = Self::build_about_page();

        stack.add_titled(&general_page, Some("general"), "General");
        stack.add_titled(&timers_page, Some("timers"), "Timers");
        stack.add_titled(&messages_page, Some("messages"), "Messages");
        stack.add_titled(&appearance_page, Some("appearance"), "Appearance");
        stack.add_titled(&about_page, Some("about"), "About");

        // Connect sidebar to stack
        let stack_clone = stack.clone();
        sidebar.connect_row_selected(move |_, row| {
            if let Some(row) = row {
                let index = row.index();
                #[allow(clippy::match_same_arms)]
                let page_name = match index {
                    0 => "general",
                    1 => "timers",
                    2 => "messages",
                    3 => "appearance",
                    4 => "about",
                    _ => "general",
                };
                stack_clone.set_visible_child_name(page_name);
            }
        });

        // Select first row by default
        if let Some(first_row) = sidebar.row_at_index(0) {
            sidebar.select_row(Some(&first_row));
        }

        // Add sidebar and stack to content box
        let separator = gtk4::Separator::new(Orientation::Vertical);
        content_box.append(&sidebar);
        content_box.append(&separator);
        content_box.append(&stack);

        main_box.append(&content_box);

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

    /// Create sidebar navigation
    fn create_sidebar() -> ListBox {
        let sidebar = ListBox::new();
        sidebar.add_css_class("navigation-sidebar");

        let general_row = adw::ActionRow::new();
        general_row.set_title("General");
        general_row.set_icon_name(Some("preferences-system-symbolic"));
        sidebar.append(&general_row);

        let timers_row = adw::ActionRow::new();
        timers_row.set_title("Timers");
        timers_row.set_icon_name(Some("alarm-symbolic"));
        sidebar.append(&timers_row);

        let messages_row = adw::ActionRow::new();
        messages_row.set_title("Messages");
        messages_row.set_icon_name(Some("mail-message-symbolic"));
        sidebar.append(&messages_row);

        let appearance_row = adw::ActionRow::new();
        appearance_row.set_title("Appearance");
        appearance_row.set_icon_name(Some("preferences-desktop-wallpaper-symbolic"));
        sidebar.append(&appearance_row);

        let about_row = adw::ActionRow::new();
        about_row.set_title("About");
        about_row.set_icon_name(Some("help-about-symbolic"));
        sidebar.append(&about_row);

        sidebar
    }

    /// Build General settings page
    fn build_general_page(&self, config: &crate::config::Config) -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let prefs_page = adw::PreferencesPage::new();
        prefs_page.set_margin_top(24);
        prefs_page.set_margin_bottom(24);
        prefs_page.set_margin_start(24);
        prefs_page.set_margin_end(24);

        let general_group = adw::PreferencesGroup::new();
        general_group.set_title("General Settings");
        general_group.set_margin_bottom(24);

        // Enable/disable breaks
        let enabled_row = adw::ActionRow::new();
        enabled_row.set_title("Enable Break Reminders");
        enabled_row.set_subtitle("Turn break notifications on or off");
        let enabled_switch = Switch::new();
        enabled_switch.set_active(config.enabled);
        enabled_switch.set_valign(Align::Center);
        enabled_row.add_suffix(&enabled_switch);
        enabled_row.set_activatable_widget(Some(&enabled_switch));
        general_group.add(&enabled_row);

        // Auto-start
        let autostart_row = adw::ActionRow::new();
        autostart_row.set_title("Start on Login");
        autostart_row.set_subtitle("Automatically start Lookout when you log in");
        let autostart_switch = Switch::new();
        autostart_switch.set_active(config.auto_start);
        autostart_switch.set_valign(Align::Center);
        autostart_row.add_suffix(&autostart_switch);
        autostart_row.set_activatable_widget(Some(&autostart_switch));
        general_group.add(&autostart_row);

        prefs_page.add(&general_group);

        // Save button in its own group
        let button_group = adw::PreferencesGroup::new();
        button_group.set_margin_top(12);

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::End);

        let save_button = Button::with_label("Save Settings");
        save_button.add_css_class("suggested-action");
        save_button.set_margin_top(8);
        save_button.set_margin_bottom(8);

        let settings_clone = self.settings.clone();
        save_button.connect_clicked(move |_| {
            let new_auto_start = autostart_switch.is_active();

            if let Err(e) = settings_clone.update(|cfg| {
                cfg.enabled = enabled_switch.is_active();
                cfg.auto_start = new_auto_start;
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
        prefs_page.add(&button_group);

        scrolled.set_child(Some(&prefs_page));
        scrolled
    }

    /// Build Timers settings page
    fn build_timers_page(&self, config: &crate::config::Config) -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let prefs_page = adw::PreferencesPage::new();
        prefs_page.set_margin_top(24);
        prefs_page.set_margin_bottom(24);
        prefs_page.set_margin_start(24);
        prefs_page.set_margin_end(24);

        // Micro break group
        let micro_group = adw::PreferencesGroup::new();
        micro_group.set_title("Micro Break");
        micro_group.set_description(Some("Short breaks to reduce eye strain (20-20-20 rule)"));
        micro_group.set_margin_bottom(24);

        let micro_interval_row = adw::ActionRow::new();
        micro_interval_row.set_title("Interval");
        micro_interval_row.set_subtitle("Minutes between micro breaks");
        let micro_interval_spin = SpinButton::with_range(1.0, 120.0, 1.0);
        micro_interval_spin.set_value(f64::from(config.micro_break_interval_minutes));
        micro_interval_spin.set_valign(Align::Center);
        micro_interval_row.add_suffix(&micro_interval_spin);
        micro_group.add(&micro_interval_row);

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
        long_group.set_margin_bottom(24);

        let long_interval_row = adw::ActionRow::new();
        long_interval_row.set_title("Interval");
        long_interval_row.set_subtitle("Minutes between long breaks");
        let long_interval_spin = SpinButton::with_range(15.0, 480.0, 5.0);
        long_interval_spin.set_value(f64::from(config.long_break_interval_minutes));
        long_interval_spin.set_valign(Align::Center);
        long_interval_row.add_suffix(&long_interval_spin);
        long_group.add(&long_interval_row);

        let long_duration_row = adw::ActionRow::new();
        long_duration_row.set_title("Duration");
        long_duration_row.set_subtitle("Minutes for each long break");
        let long_duration_spin = SpinButton::with_range(1.0, 60.0, 1.0);
        long_duration_spin.set_value(f64::from(config.long_break_duration_minutes));
        long_duration_spin.set_valign(Align::Center);
        long_duration_row.add_suffix(&long_duration_spin);
        long_group.add(&long_duration_row);

        prefs_page.add(&long_group);

        // Save button
        let button_group = adw::PreferencesGroup::new();
        button_group.set_margin_top(12);

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::End);

        let save_button = Button::with_label("Save Settings");
        save_button.add_css_class("suggested-action");
        save_button.set_margin_top(8);
        save_button.set_margin_bottom(8);

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
        prefs_page.add(&button_group);

        scrolled.set_child(Some(&prefs_page));
        scrolled
    }

    /// Build Messages management page
    fn build_messages_page(config: &crate::config::Config) -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let prefs_page = adw::PreferencesPage::new();
        prefs_page.set_margin_top(24);
        prefs_page.set_margin_bottom(24);
        prefs_page.set_margin_start(24);
        prefs_page.set_margin_end(24);

        // Info note at top
        let info_group = adw::PreferencesGroup::new();
        info_group.set_margin_bottom(24);
        let info_row = adw::ActionRow::new();
        info_row.set_title("Random Message Selection");
        info_row.set_subtitle("One message from each group is randomly selected every break");
        info_group.add(&info_row);
        prefs_page.add(&info_group);

        // Headings group
        let headings_group = adw::PreferencesGroup::new();
        headings_group.set_title("Break Headings");
        headings_group.set_description(Some("Main messages shown at the top of break screens"));
        headings_group.set_margin_bottom(24);

        for (i, heading) in config.break_messages.headings.iter().enumerate() {
            let row = adw::ActionRow::new();
            row.set_title(&format!("{}. {}", i + 1, heading));
            headings_group.add(&row);
        }

        prefs_page.add(&headings_group);

        // Micro break instructions
        let micro_group = adw::PreferencesGroup::new();
        micro_group.set_title("Micro Break Messages");
        micro_group.set_description(Some("Instructions for 20-second eye rest breaks"));
        micro_group.set_margin_bottom(24);

        for (i, instruction) in config
            .break_messages
            .micro_break_instructions
            .iter()
            .enumerate()
        {
            let row = adw::ActionRow::new();
            row.set_title(&format!("{}. {}", i + 1, instruction));
            micro_group.add(&row);
        }

        prefs_page.add(&micro_group);

        // Long break instructions
        let long_group = adw::PreferencesGroup::new();
        long_group.set_title("Long Break Messages");
        long_group.set_description(Some("Instructions for 5-minute movement breaks"));
        long_group.set_margin_bottom(24);

        for (i, instruction) in config
            .break_messages
            .long_break_instructions
            .iter()
            .enumerate()
        {
            let row = adw::ActionRow::new();
            row.set_title(&format!("{}. {}", i + 1, instruction));
            long_group.add(&row);
        }

        prefs_page.add(&long_group);

        // Note about editing
        let note_group = adw::PreferencesGroup::new();
        note_group.set_margin_top(12);
        let note_row = adw::ActionRow::new();
        note_row.set_title("Note");
        note_row.set_subtitle(
            "Message editing via UI is coming soon. You can manually edit ~/.config/lookout/config.json",
        );
        note_group.add(&note_row);
        prefs_page.add(&note_group);

        scrolled.set_child(Some(&prefs_page));
        scrolled
    }

    /// Build Appearance settings page
    #[allow(clippy::too_many_lines)]
    fn build_appearance_page(&self, config: &crate::config::Config) -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let prefs_page = adw::PreferencesPage::new();
        prefs_page.set_margin_top(24);
        prefs_page.set_margin_bottom(24);
        prefs_page.set_margin_start(24);
        prefs_page.set_margin_end(24);

        let appearance_group = adw::PreferencesGroup::new();
        appearance_group.set_title("Break Window Colors");
        appearance_group.set_description(Some("Customize the fullscreen break overlay"));
        appearance_group.set_margin_bottom(24);

        // Background color
        let bg_color_row = adw::ActionRow::new();
        bg_color_row.set_title("Background Color");
        bg_color_row.set_subtitle("Click to choose a background color");

        let bg_color_button = ColorButton::new();
        if let Some(rgba) = parse_color_string(&config.background_color) {
            bg_color_button.set_rgba(&rgba);
        }
        bg_color_button.set_valign(Align::Center);
        bg_color_row.add_suffix(&bg_color_button);
        appearance_group.add(&bg_color_row);

        // Text color
        let text_color_row = adw::ActionRow::new();
        text_color_row.set_title("Text Color");
        text_color_row.set_subtitle("Click to choose a text color");

        let text_color_button = ColorButton::new();
        if let Some(rgba) = parse_color_string(&config.text_color) {
            text_color_button.set_rgba(&rgba);
        }
        text_color_button.set_valign(Align::Center);
        text_color_row.add_suffix(&text_color_button);
        appearance_group.add(&text_color_row);

        prefs_page.add(&appearance_group);

        // Presets
        let presets_group = adw::PreferencesGroup::new();
        presets_group.set_title("Quick Presets");
        presets_group.set_description(Some("Apply preset color combinations"));
        presets_group.set_margin_bottom(24);

        let presets = vec![
            ("Default Dark", "rgba(0, 0, 0, 0.95)", "rgba(255, 255, 255, 1.0)", "Black background with white text"),
            ("Pure Black", "rgba(0, 0, 0, 1.0)", "rgba(255, 255, 255, 1.0)", "Solid black with white text"),
            ("Navy Blue", "rgba(15, 23, 42, 0.95)", "rgba(226, 232, 240, 1.0)", "Dark blue with light gray text"),
            ("Deep Purple", "rgba(30, 20, 60, 0.95)", "rgba(243, 232, 255, 1.0)", "Purple background with lavender text"),
        ];

        for (name, bg_color, text_color, description) in presets {
            let preset_row = adw::ActionRow::new();
            preset_row.set_title(name);
            preset_row.set_subtitle(description);

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

        prefs_page.add(&presets_group);

        // Buttons
        let button_group = adw::PreferencesGroup::new();
        button_group.set_margin_top(12);

        let button_box = GtkBox::new(Orientation::Horizontal, 12);
        button_box.set_halign(Align::End);

        // Preview button
        let preview_button = Button::with_label("Preview");
        preview_button.set_margin_top(8);
        preview_button.set_margin_bottom(8);

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
        save_button.set_margin_top(8);
        save_button.set_margin_bottom(8);

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
        prefs_page.add(&button_group);

        scrolled.set_child(Some(&prefs_page));
        scrolled
    }

    /// Build About page
    fn build_about_page() -> ScrolledWindow {
        let scrolled = ScrolledWindow::new();
        scrolled.set_vexpand(true);

        let prefs_page = adw::PreferencesPage::new();
        prefs_page.set_margin_top(24);
        prefs_page.set_margin_bottom(24);
        prefs_page.set_margin_start(24);
        prefs_page.set_margin_end(24);

        let about_group = adw::PreferencesGroup::new();
        about_group.set_title("About Lookout");
        about_group.set_description(Some("Break reminder application for Linux"));
        about_group.set_margin_bottom(24);

        let version_row = adw::ActionRow::new();
        version_row.set_title("Version");
        version_row.set_subtitle("0.1.0");
        about_group.add(&version_row);

        let desc_row = adw::ActionRow::new();
        desc_row.set_title("Description");
        desc_row.set_subtitle(
            "A lightweight break reminder app to reduce eye strain and promote healthy breaks",
        );
        about_group.add(&desc_row);

        prefs_page.add(&about_group);

        let dev_group = adw::PreferencesGroup::new();
        dev_group.set_title("Development");
        dev_group.set_margin_bottom(24);

        let ai_row = adw::ActionRow::new();
        ai_row.set_title("Built with AI");
        ai_row.set_subtitle("Developed and maintained entirely using Claude Code (AI)");
        dev_group.add(&ai_row);

        prefs_page.add(&dev_group);

        let links_group = adw::PreferencesGroup::new();
        links_group.set_title("Links & Resources");
        links_group.set_margin_bottom(24);

        let github_row = adw::ActionRow::new();
        github_row.set_title("GitHub Repository");
        github_row.set_subtitle("https://github.com/raghulj/lookout");
        links_group.add(&github_row);

        let license_row = adw::ActionRow::new();
        license_row.set_title("License");
        license_row.set_subtitle("MIT License - Free and open source");
        links_group.add(&license_row);

        prefs_page.add(&links_group);

        scrolled.set_child(Some(&prefs_page));
        scrolled
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
    if let Some(inner) = color_str.strip_prefix("rgba(").and_then(|s| s.strip_suffix(')')) {
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
