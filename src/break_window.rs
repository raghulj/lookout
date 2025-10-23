//! Break overlay window using GTK4

use crate::config::Config;
use crate::timer::BreakType;
use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Box as GtkBox, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Break overlay window
pub struct BreakWindow {
    window: Rc<RefCell<Option<ApplicationWindow>>>,
    config: Config,
}

impl BreakWindow {
    /// Create a new break window
    pub fn new(config: Config) -> Self {
        Self {
            window: Rc::new(RefCell::new(None)),
            config,
        }
    }

    /// Show the break overlay
    #[allow(clippy::too_many_lines)]
    pub fn show(&self, break_type: BreakType, duration: Duration) {
        log::info!("Showing break overlay: {break_type:?} for {duration:?}");

        // Create fullscreen window
        let window = ApplicationWindow::builder()
            .title("Lookout - Break Time")
            .fullscreened(true)
            .decorated(false)
            .build();

        // Add CSS class for styling
        window.add_css_class("break-window");

        // Apply dynamic background color from config
        let css_provider = gtk4::CssProvider::new();
        let background_css = format!(
            ".break-window {{ background: {}; }}",
            self.config.background_color
        );
        css_provider.load_from_data(&background_css);

        window
            .style_context()
            .add_provider(&css_provider, gtk4::STYLE_PROVIDER_PRIORITY_USER);

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.set_halign(Align::Fill);
        main_box.set_valign(Align::Fill);

        // Top bar with current time
        let top_box = GtkBox::new(Orientation::Horizontal, 0);
        top_box.set_halign(Align::Center);
        top_box.set_margin_top(32);

        let time_label = Label::new(None);
        time_label.add_css_class("current-time");
        update_current_time(&time_label);
        top_box.append(&time_label);

        // Update time every second
        gtk4::glib::timeout_add_seconds_local(1, move || {
            update_current_time(&time_label);
            gtk4::glib::ControlFlow::Continue
        });

        main_box.append(&top_box);

        // Center content box
        let center_box = GtkBox::new(Orientation::Vertical, 32);
        center_box.set_halign(Align::Center);
        center_box.set_valign(Align::Center);
        center_box.set_vexpand(true);

        // Main heading - random selection
        let heading = self.config.break_messages.random_heading();
        let heading_label = Label::new(Some(heading));
        heading_label.add_css_class("break-heading");
        center_box.append(&heading_label);

        // Instruction message - random selection based on break type
        let instruction = match break_type {
            BreakType::Micro => self.config.break_messages.random_micro_instruction(),
            BreakType::Long => self.config.break_messages.random_long_instruction(),
        };

        let instruction_label = Label::new(Some(instruction));
        instruction_label.add_css_class("break-instruction");
        instruction_label.set_wrap(true);
        instruction_label.set_max_width_chars(60);
        instruction_label.set_justify(gtk4::Justification::Center);
        center_box.append(&instruction_label);

        // Countdown timer with flip animation
        let timer_box = GtkBox::new(Orientation::Horizontal, 12);
        timer_box.set_halign(Align::Center);
        timer_box.add_css_class("countdown-container");

        let remaining_secs = Rc::new(RefCell::new(duration.as_secs()));

        // Create individual labels for MM:SS format
        let minute_tens = Label::new(Some("0"));
        minute_tens.add_css_class("break-countdown");
        minute_tens.add_css_class("countdown-digit");

        let minute_ones = Label::new(Some("0"));
        minute_ones.add_css_class("break-countdown");
        minute_ones.add_css_class("countdown-digit");

        let colon_label = Label::new(Some(":"));
        colon_label.add_css_class("break-countdown");
        colon_label.add_css_class("countdown-separator");

        let second_tens = Label::new(Some("0"));
        second_tens.add_css_class("break-countdown");
        second_tens.add_css_class("countdown-digit");

        let second_ones = Label::new(Some("0"));
        second_ones.add_css_class("break-countdown");
        second_ones.add_css_class("countdown-digit");

        // Set initial values
        update_countdown_digits(
            &minute_tens,
            &minute_ones,
            &second_tens,
            &second_ones,
            duration,
        );

        timer_box.append(&minute_tens);
        timer_box.append(&minute_ones);
        timer_box.append(&colon_label);
        timer_box.append(&second_tens);
        timer_box.append(&second_ones);

        center_box.append(&timer_box);

        // Bottom buttons
        let button_box = GtkBox::new(Orientation::Horizontal, 16);
        button_box.set_halign(Align::Center);
        button_box.set_margin_top(32);

        // Skip button (with chevron icon)
        let skip_button = Button::with_label("⏩ Skip");
        skip_button.add_css_class("break-skip-button");

        button_box.append(&skip_button);

        center_box.append(&button_box);

        // Bottom hint text
        let hint_label = Label::new(Some("Press Esc twice to skip"));
        hint_label.add_css_class("break-hint");
        hint_label.set_margin_bottom(32);
        hint_label.set_margin_top(24);
        center_box.append(&hint_label);

        main_box.append(&center_box);

        // Update countdown every second
        let minute_tens_weak = minute_tens.downgrade();
        let minute_ones_weak = minute_ones.downgrade();
        let second_tens_weak = second_tens.downgrade();
        let second_ones_weak = second_ones.downgrade();
        let window_clone = window.clone();

        gtk4::glib::timeout_add_seconds_local(1, move || {
            let mut secs = remaining_secs.borrow_mut();
            if *secs == 0 {
                window_clone.close();
                return gtk4::glib::ControlFlow::Break;
            }

            *secs -= 1;
            let duration = Duration::from_secs(*secs);

            // Update all digit labels with animation
            if let (Some(mt), Some(mo), Some(st), Some(so)) = (
                minute_tens_weak.upgrade(),
                minute_ones_weak.upgrade(),
                second_tens_weak.upgrade(),
                second_ones_weak.upgrade(),
            ) {
                update_countdown_digits(&mt, &mo, &st, &so, duration);
            }

            gtk4::glib::ControlFlow::Continue
        });

        // Skip button handler
        let window_clone = window.clone();
        skip_button.connect_clicked(move |_| {
            log::info!("Break skipped by user");
            window_clone.close();
        });

        // Double ESC key handler
        let esc_press_time = Rc::new(RefCell::new(None::<std::time::Instant>));
        let window_clone = window.clone();

        let key_controller = gtk4::EventControllerKey::new();
        let esc_press_time_clone = Rc::clone(&esc_press_time);

        key_controller.connect_key_pressed(move |_, key, _code, _modifier| {
            if key == gtk4::gdk::Key::Escape {
                let now = std::time::Instant::now();
                let mut last_press = esc_press_time_clone.borrow_mut();

                if let Some(last_time) = *last_press {
                    // Check if second ESC is within 1 second
                    if now.duration_since(last_time).as_secs() < 1 {
                        log::info!("Double ESC detected - skipping break");
                        window_clone.close();
                        return gtk4::glib::Propagation::Stop;
                    }
                }

                *last_press = Some(now);
            }
            gtk4::glib::Propagation::Proceed
        });

        window.add_controller(key_controller);

        window.set_child(Some(&main_box));

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

    /// Hide the break overlay
    #[allow(dead_code)]
    pub fn hide(&self) {
        log::info!("Hiding break overlay");
        if let Some(window) = self.window.borrow().as_ref() {
            window.close();
        }
    }
}

impl Default for BreakWindow {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

/// Update current time label
fn update_current_time(label: &Label) {
    use std::time::SystemTime;

    let now = SystemTime::now();
    let datetime = chrono::DateTime::<chrono::Local>::from(now);
    let time_str = datetime.format("Current time is %H:%M").to_string();
    label.set_text(&time_str);
}

/// Update countdown digit labels
fn update_countdown_digits(
    minute_tens: &Label,
    minute_ones: &Label,
    second_tens: &Label,
    second_ones: &Label,
    duration: Duration,
) {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;

    let min_tens = minutes / 10;
    let min_ones = minutes % 10;
    let sec_tens = seconds / 10;
    let sec_ones = seconds % 10;

    minute_tens.set_text(&min_tens.to_string());
    minute_ones.set_text(&min_ones.to_string());
    second_tens.set_text(&sec_tens.to_string());
    second_ones.set_text(&sec_ones.to_string());
}
