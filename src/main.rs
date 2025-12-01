mod app;
mod autostart;
mod break_window;
mod config;
mod idle;
mod settings;
mod settings_window;
mod timer;
mod tray;
mod updater;

use std::env;
use std::error::Error;
use std::fs;
use std::path::PathBuf;

const VERSION: &str = env!("CARGO_PKG_VERSION");
const REPO_URL: &str = "https://github.com/raghulj/lookout";

fn get_autostart_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".config/autostart/lookout.desktop")
}

fn get_desktop_entry_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".local/share/applications/lookout.desktop")
}

#[allow(dead_code)]
fn get_install_path() -> PathBuf {
    let home = env::var("HOME").expect("HOME environment variable not set");
    PathBuf::from(home).join(".local/bin/lookout")
}

fn enable_autostart() -> Result<(), Box<dyn Error>> {
    let autostart_dir = get_autostart_path().parent().unwrap().to_path_buf();
    let desktop_entry = get_desktop_entry_path();
    let autostart_file = get_autostart_path();

    // Create autostart directory if it doesn't exist
    fs::create_dir_all(&autostart_dir)?;

    // Copy desktop entry to autostart
    if desktop_entry.exists() {
        fs::copy(&desktop_entry, &autostart_file)?;
        println!("✅ Autostart enabled");
        println!("   Lookout will start automatically on login");
    } else {
        eprintln!("❌ Desktop entry not found: {}", desktop_entry.display());
        eprintln!("   Please reinstall Lookout using the install script");
        return Err("Desktop entry not found".into());
    }

    Ok(())
}

fn disable_autostart() -> Result<(), Box<dyn Error>> {
    let autostart_file = get_autostart_path();

    if autostart_file.exists() {
        fs::remove_file(&autostart_file)?;
        println!("✅ Autostart disabled");
    } else {
        println!("ℹ️  Autostart was not enabled");
    }

    Ok(())
}

fn show_version() {
    println!("Lookout v{}", VERSION);
    println!("Break Reminder for Linux");
    println!();
    println!("Repository: {}", REPO_URL);
}

fn show_help() {
    println!("Lookout v{} - Break Reminder for Linux", VERSION);
    println!();
    println!("USAGE:");
    println!("    lookout [OPTIONS]");
    println!();
    println!("OPTIONS:");
    println!("    --version                Show version information");
    println!("    --help                   Show this help message");
    println!("    --enable-autostart       Enable autostart on login");
    println!("    --disable-autostart      Disable autostart on login");
    println!();
    println!("INSTALLATION:");
    println!("    Install: curl -fsSL https://raw.githubusercontent.com/raghulj/lookout/main/install.sh | bash");
    println!("    Update:  curl -fsSL https://raw.githubusercontent.com/raghulj/lookout/main/install.sh | bash");
    println!("    Remove:  curl -fsSL https://raw.githubusercontent.com/raghulj/lookout/main/uninstall.sh | bash");
    println!();
    println!("CONFIGURATION:");
    println!("    Settings: ~/.config/lookout/config.json");
    println!();
    println!("For more information, visit: {}", REPO_URL);
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();

    if args.len() > 1 {
        match args[1].as_str() {
            "--version" | "-v" => {
                show_version();
                return Ok(());
            },
            "--help" | "-h" => {
                show_help();
                return Ok(());
            },
            "--enable-autostart" => {
                return enable_autostart();
            },
            "--disable-autostart" => {
                return disable_autostart();
            },
            _ => {
                eprintln!("Unknown option: {}", args[1]);
                eprintln!("Run 'lookout --help' for usage information");
                return Err("Unknown option".into());
            },
        }
    }

    // Initialize logger
    env_logger::init();

    log::info!("Starting Lookout v{}", VERSION);

    // Initialize GTK application
    let app = app::LookoutApp::new();

    // Run the application
    app.run()
}
