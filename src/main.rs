mod app;
mod break_window;
mod config;
mod settings;
mod settings_window;
mod timer;
mod tray;

use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Initialize logger
    env_logger::init();

    log::info!("Starting Lookout application...");

    // Initialize GTK application
    let app = app::LookoutApp::new();

    // Run the application
    app.run()
}
