//! Break overlay window using GTK4

use crate::config::Config;
use crate::timer::BreakType;
use gtk4::glib;
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

        // Set window background color using native GTK
        // Parse RGBA color from config
        let bg_color = parse_rgba(&self.config.background_color);
        let text_color = rgba_to_pango_color(&self.config.text_color);

        // Create a drawing area to paint the background
        let drawing_area = gtk4::DrawingArea::new();
        drawing_area.set_draw_func(move |_, cr, width, height| {
            cr.set_source_rgba(bg_color.0, bg_color.1, bg_color.2, bg_color.3);
            let _ = cr.rectangle(0.0, 0.0, width as f64, height as f64);
            let _ = cr.fill();
        });

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 0);
        main_box.set_halign(Align::Fill);
        main_box.set_valign(Align::Fill);

        // Top bar with current time
        let top_box = GtkBox::new(Orientation::Horizontal, 0);
        top_box.set_halign(Align::Center);
        top_box.set_margin_top(32);

        let time_label = Label::new(None);
        let text_color_for_time = text_color.clone();
        update_current_time(&time_label, &text_color_for_time);
        top_box.append(&time_label);

        // Update time every second
        let text_color_for_timer = text_color.clone();
        gtk4::glib::timeout_add_seconds_local(1, move || {
            update_current_time(&time_label, &text_color_for_timer);
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
        heading_label.set_markup(&format!(
            "<span size='56000' weight='bold' foreground='{}'>{}</span>",
            text_color,
            glib::markup_escape_text(heading)
        ));
        center_box.append(&heading_label);

        // Instruction message - random selection based on break type
        let instruction = match break_type {
            BreakType::Micro => self.config.break_messages.random_micro_instruction(),
            BreakType::Long => self.config.break_messages.random_long_instruction(),
        };

        let instruction_label = Label::new(Some(instruction));
        instruction_label.set_markup(&format!(
            "<span size='20000' foreground='{}'>{}</span>",
            text_color,
            glib::markup_escape_text(instruction)
        ));
        instruction_label.set_wrap(true);
        instruction_label.set_max_width_chars(60);
        instruction_label.set_justify(gtk4::Justification::Center);
        center_box.append(&instruction_label);

        // Countdown timer with flip animation
        let timer_box = GtkBox::new(Orientation::Horizontal, 12);
        timer_box.set_halign(Align::Center);

        let remaining_secs = Rc::new(RefCell::new(duration.as_secs()));

        // Create individual labels for MM:SS format
        let minute_tens = Label::new(Some("0"));
        let minute_ones = Label::new(Some("0"));
        let colon_label = Label::new(Some(":"));
        let second_tens = Label::new(Some("0"));
        let second_ones = Label::new(Some("0"));

        // Set initial values
        let text_color_for_countdown = text_color.clone();
        update_countdown_digits(
            &minute_tens,
            &minute_ones,
            &second_tens,
            &second_ones,
            duration,
            &text_color_for_countdown,
        );

        // Set colon color
        colon_label.set_markup(&format!(
            "<span size='120000' foreground='{}'>:</span>",
            text_color
        ));

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
        skip_button.set_size_request(140, 48);

        button_box.append(&skip_button);

        center_box.append(&button_box);

        // Bottom hint text
        let hint_label = Label::new(Some("Press Esc twice to skip"));
        // Create a dimmed version of the text color for hints
        let hint_color = dim_rgba_color(&self.config.text_color, 0.6);
        hint_label.set_markup(&format!(
            "<span size='14000' foreground='{}'>Press Esc twice to skip</span>",
            hint_color
        ));
        hint_label.set_margin_bottom(32);
        hint_label.set_margin_top(24);
        center_box.append(&hint_label);

        main_box.append(&center_box);

        // Create overlay to have drawing area as background
        let overlay = gtk4::Overlay::new();
        overlay.set_child(Some(&drawing_area));
        overlay.add_overlay(&main_box);

        // Update countdown every second
        let minute_tens_weak = minute_tens.downgrade();
        let minute_ones_weak = minute_ones.downgrade();
        let second_tens_weak = second_tens.downgrade();
        let second_ones_weak = second_ones.downgrade();
        let window_clone = window.clone();
        let text_color_for_update = text_color.clone();

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
                update_countdown_digits(&mt, &mo, &st, &so, duration, &text_color_for_update);
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

        window.set_child(Some(&overlay));

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
fn update_current_time(label: &Label, text_color: &str) {
    use std::time::SystemTime;

    let now = SystemTime::now();
    let datetime = chrono::DateTime::<chrono::Local>::from(now);
    let time_str = datetime.format("Current time is %H:%M").to_string();
    label.set_markup(&format!(
        "<span size='16000' weight='500' foreground='{}'>{}</span>",
        text_color,
        glib::markup_escape_text(&time_str)
    ));
}

/// Update countdown digit labels
fn update_countdown_digits(
    minute_tens: &Label,
    minute_ones: &Label,
    second_tens: &Label,
    second_ones: &Label,
    duration: Duration,
    text_color: &str,
) {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;

    let min_tens = minutes / 10;
    let min_ones = minutes % 10;
    let sec_tens = seconds / 10;
    let sec_ones = seconds % 10;

    minute_tens.set_markup(&format!(
        "<span size='120000' foreground='{}'>{}</span>",
        text_color, min_tens
    ));
    minute_ones.set_markup(&format!(
        "<span size='120000' foreground='{}'>{}</span>",
        text_color, min_ones
    ));
    second_tens.set_markup(&format!(
        "<span size='120000' foreground='{}'>{}</span>",
        text_color, sec_tens
    ));
    second_ones.set_markup(&format!(
        "<span size='120000' foreground='{}'>{}</span>",
        text_color, sec_ones
    ));
}

/// Parse RGBA color string to tuple (r, g, b, a) for Cairo
fn parse_rgba(color_str: &str) -> (f64, f64, f64, f64) {
    // Default to black with 95% opacity
    let default = (0.0, 0.0, 0.0, 0.95);

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
                parts[3].parse::<f64>(),
            ) {
                return (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0, a);
            }
        }
    }

    default
}

/// Convert RGBA color string to Pango color format for markup
/// Pango markup accepts #RRGGBB hex format, not rgba()
fn rgba_to_pango_color(color_str: &str) -> String {
    // Try to parse rgba(r, g, b, a) format
    if let Some(inner) = color_str
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 4 {
            if let (Ok(r), Ok(g), Ok(b), Ok(_a)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
                parts[3].parse::<f64>(),
            ) {
                // Pango accepts #RRGGBB hex format (alpha is handled separately with alpha attribute)
                return format!("#{:02X}{:02X}{:02X}", r, g, b);
            }
        }
    }

    // Default to white if parsing fails
    "#FFFFFF".to_string()
}

/// Create a dimmed version of an RGBA color
/// For Pango, we blend the color with the background for a dimming effect
fn dim_rgba_color(color_str: &str, dim_factor: f64) -> String {
    // Try to parse rgba(r, g, b, a) format
    if let Some(inner) = color_str
        .strip_prefix("rgba(")
        .and_then(|s| s.strip_suffix(')'))
    {
        let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();
        if parts.len() == 4 {
            if let (Ok(r), Ok(g), Ok(b), Ok(_a)) = (
                parts[0].parse::<u8>(),
                parts[1].parse::<u8>(),
                parts[2].parse::<u8>(),
                parts[3].parse::<f64>(),
            ) {
                // Dim the color by reducing its intensity
                let dimmed_r = (f64::from(r) * dim_factor) as u8;
                let dimmed_g = (f64::from(g) * dim_factor) as u8;
                let dimmed_b = (f64::from(b) * dim_factor) as u8;
                return format!("#{:02X}{:02X}{:02X}", dimmed_r, dimmed_g, dimmed_b);
            }
        }
    }

    // Default to dimmed white if parsing fails
    "#999999".to_string()
}
