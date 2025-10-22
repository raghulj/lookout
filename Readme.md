# Lookout App for Linux

A lightweight, native break reminder application for Linux that helps reduce eye strain and digital fatigue through micro breaks and long breaks.

## Project Overview

A simple, memory-efficient break reminder app that runs in the system tray and provides fullscreen break overlays. Designed to work across all major Linux desktop environments with both X11 and Wayland support.

## Core Features

### Must-Have Features
- **Two Break Types**
  - Micro breaks (e.g., 20-second breaks every 20 minutes)
  - Long breaks (e.g., 5-minute breaks every 60 minutes)
  
- **System Tray Integration**
  - Runs in background with system tray icon
  - Shows next break time
  - Right-click menu for quick actions
  
- **Break Overlays**
  - Fullscreen semi-transparent overlay during breaks
  - Countdown timer display
  - Skip button (optional)
  
- **Settings Window**
  - Configure micro break interval and duration
  - Configure long break interval and duration
  - Start/Pause/Resume functionality
  - Auto-start on system boot option

### Design Requirements
- Minimal, clean UI following modern Linux design principles
- Low memory footprint (target: <10MB idle)
- Native look and feel on Linux
- Smooth animations and transitions

## Technical Stack

### Core Technologies
- **Language**: Rust
- **UI Framework**: GTK4
- **Styling**: libadwaita (for modern GNOME aesthetic)
- **Async Runtime**: tokio
- **System Tray**: ksni (KDE Status Notifier Item)

### Key Dependencies

```toml
[dependencies]
gtk4 = "0.9"
libadwaita = "0.7"
tokio = { version = "1.0", features = ["full"] }
ksni = "0.2"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
dirs = "5.0"
```

## Architecture

### Application Components

1. **System Tray Service**
   - Manages tray icon and menu
   - Displays next break countdown
   - Provides quick actions (pause, settings, quit)

2. **Timer Engine**
   - Tokio-based async timers
   - Tracks micro break and long break intervals
   - Handles pause/resume state

3. **Settings Window (GTK4)**
   - Configuration UI for break intervals
   - Preferences management
   - Persistent settings storage (JSON in `~/.config/lookout/`)

4. **Break Overlay Window (GTK4)**
   - Fullscreen window with semi-transparent background
   - Countdown display
   - Skip button with minimum wait time
   - Auto-dismisses when break completes

### Display Server Support

GTK4 automatically handles both X11 and Wayland:
- No conditional compilation needed
- GTK4 detects display server at runtime
- Single codebase works on both

### Desktop Environment Compatibility

**Fully Supported (System Tray Works Out of Box):**
- KDE Plasma
- XFCE
- MATE
- Cinnamon
- LXQt
- Tiling WMs with compatible status bars (i3, Sway with waybar, etc.)

**GNOME Support:**
- Requires AppIndicator extension
- Document this requirement in installation instructions
- Consider detecting GNOME and showing one-time notification

## Memory & Performance Targets

- **Idle in tray**: 3-7 MB
- **With settings window open**: 8-12 MB
- **During break overlay**: 8-12 MB
- **Binary size**: ~5-8 MB (stripped)

## File Structure

```
lookout/
├── src/
│   ├── main.rs              # Entry point
│   ├── app.rs               # GTK Application setup
│   ├── tray.rs              # System tray implementation
│   ├── timer.rs             # Break timer logic
│   ├── settings.rs          # Settings management
│   ├── settings_window.rs   # Settings UI
│   ├── break_window.rs      # Break overlay UI
│   └── config.rs            # Configuration persistence
├── resources/
│   ├── icons/               # App and tray icons
│   └── style.css            # Custom GTK styles
├── Cargo.toml
└── README.md
```

## Configuration Storage

Settings stored in: `~/.config/lookout/config.json`

```json
{
  "micro_break_interval_minutes": 20,
  "micro_break_duration_seconds": 20,
  "long_break_interval_minutes": 60,
  "long_break_duration_minutes": 5,
  "auto_start": true,
  "enabled": true
}
```

## User Experience Flow

### First Run
1. App starts and creates tray icon
2. Shows welcome notification with quick settings overview
3. Begins timer countdown
4. If on GNOME without AppIndicator, show one-time setup instruction

### During Use
1. Tray icon shows next break time on hover
2. Icon changes color/style as break approaches (optional)
3. Break window appears fullscreen at scheduled time
4. User waits for break or skips (after minimum time)
5. App returns to background

### Tray Menu Options
- "Next break in X minutes" (informational)
- "Pause breaks" / "Resume breaks"
- "Settings"
- "About"
- "Quit"

## Development Guidelines

### Code Style
- Follow Rust standard formatting (rustfmt)
- Use Clippy for linting
- Comprehensive error handling (no unwrap in production code)
- Document public APIs

### Testing
- Unit tests for timer logic
- Integration tests for settings persistence
- Manual testing across multiple desktop environments

### Build Instructions

```bash
# Development build
cargo build

# Release build (optimized)
cargo build --release

# Strip binary for minimal size
strip target/release/lookout
```

## Installation & Distribution

### Package Formats
- **Primary**: AppImage (universal, no dependencies)
- **Secondary**: Flatpak (sandboxed, broader reach)
- **Optional**: .deb and .rpm packages

### Desktop Entry
Install to: `~/.local/share/applications/lookout.desktop`

```ini
[Desktop Entry]
Name=Lookout
Comment=Helps reduce eye strain with periodic breaks
Exec=/path/to/lookout
Icon=lookout
Type=Application
Categories=Utility;
StartupNotify=false
X-GNOME-Autostart-enabled=true
```

## Known Limitations

1. **GNOME System Tray**: Requires AppIndicator extension
2. **Idle Detection**: Basic version doesn't detect user idle time (future enhancement)
3. **Multi-Monitor**: Break overlay shows on primary monitor only (v1.0)

## Future Enhancements (Post-MVP)

- Idle time detection (don't start break if user already away)
- Multi-monitor support
- Break exercise suggestions/animations
- Statistics and usage insights
- Sound notifications (optional)
- Theme customization
- Pomodoro mode

## Success Criteria

- Runs reliably in background consuming <10MB RAM
- Works on Ubuntu, Fedora, Arch, and openSUSE without modification
- System tray icon visible on KDE, XFCE, MATE, Cinnamon
- Break overlays work smoothly on both X11 and Wayland
- Settings persist across restarts
- Clean, intuitive UI that follows Linux desktop conventions

## Developer Resources

- [GTK4 Rust Book](https://gtk-rs.org/gtk4-rs/stable/latest/book/)
- [libadwaita Documentation](https://gnome.pages.gitlab.gnome.org/libadwaita/)
- [ksni Crate Documentation](https://docs.rs/ksni/)
- [GNOME Human Interface Guidelines](https://developer.gnome.org/hig/)

## License

[Choose appropriate license - MIT, GPL-3.0, Apache-2.0, etc.]

## Contributing

[Add contribution guidelines if open source]

---

**Project Goal**: Create the simplest, most efficient break reminder app for Linux that just works across all desktop environments.
