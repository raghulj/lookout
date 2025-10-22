//! Break overlay window using GTK4

use crate::timer::BreakType;
use gtk4::prelude::*;
use gtk4::{Align, ApplicationWindow, Box as GtkBox, Button, Label, Orientation};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

/// Break overlay window
pub struct BreakWindow {
    window: Rc<RefCell<Option<ApplicationWindow>>>,
}

impl BreakWindow {
    /// Create a new break window
    pub fn new() -> Self {
        Self {
            window: Rc::new(RefCell::new(None)),
        }
    }

    /// Show the break overlay
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

        // Create main container
        let main_box = GtkBox::new(Orientation::Vertical, 24);
        main_box.set_halign(Align::Center);
        main_box.set_valign(Align::Center);
        main_box.set_margin_top(48);
        main_box.set_margin_bottom(48);

        // Break type message
        let break_message = match break_type {
            BreakType::Micro => "Time for a Micro Break",
            BreakType::Long => "Time for a Long Break",
        };

        let message_label = Label::new(Some(break_message));
        message_label.add_css_class("break-message");
        message_label.set_margin_bottom(24);
        main_box.append(&message_label);

        // Countdown label
        let countdown_label = Label::new(Some(&format_duration(duration)));
        countdown_label.add_css_class("break-countdown");
        main_box.append(&countdown_label);

        // Break message/instruction
        let instruction = match break_type {
            BreakType::Micro => "Look away from your screen and rest your eyes",
            BreakType::Long => "Stand up, stretch, and move around",
        };

        let instruction_label = Label::new(Some(instruction));
        instruction_label.set_margin_top(24);
        instruction_label.set_margin_bottom(48);
        main_box.append(&instruction_label);

        // Skip button (enabled after minimum time)
        let skip_button = Button::with_label("Skip Break");
        skip_button.add_css_class("break-skip-button");
        skip_button.set_sensitive(false);
        main_box.append(&skip_button);

        // Enable skip button after 5 seconds
        let skip_button_clone = skip_button.clone();
        gtk4::glib::timeout_add_seconds_local(5, move || {
            skip_button_clone.set_sensitive(true);
            gtk4::glib::ControlFlow::Break
        });

        // Update countdown every second
        let countdown_label_weak = countdown_label.downgrade();
        let window_clone = window.clone();
        let remaining_secs = Rc::new(RefCell::new(duration.as_secs()));

        gtk4::glib::timeout_add_seconds_local(1, move || {
            let mut secs = remaining_secs.borrow_mut();
            if *secs == 0 {
                window_clone.close();
                return gtk4::glib::ControlFlow::Break;
            }

            *secs -= 1;
            if let Some(label) = countdown_label_weak.upgrade() {
                label.set_text(&format_duration(Duration::from_secs(*secs)));
            }
            gtk4::glib::ControlFlow::Continue
        });

        // Skip button handler
        let window_clone = window.clone();
        skip_button.connect_clicked(move |_| {
            log::info!("Break skipped by user");
            window_clone.close();
        });

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
        Self::new()
    }
}

/// Format duration as MM:SS
fn format_duration(duration: Duration) -> String {
    let total_secs = duration.as_secs();
    let minutes = total_secs / 60;
    let seconds = total_secs % 60;
    format!("{minutes:02}:{seconds:02}")
}
