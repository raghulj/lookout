# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**Lookout** is a lightweight Linux break reminder application built with Rust, GTK4, and libadwaita. The app helps reduce eye strain through micro breaks (20s every 20min) and long breaks (5min every 60min), running as a system tray application with fullscreen break overlays.

**Memory Target**: <10MB idle
**Key Design Principle**: Minimal, native, cross-desktop-environment compatibility

## Development Commands

```bash
# Build for development
cargo build

# Build optimized release binary
cargo build --release

# Strip binary to minimize size
strip target/release/lookout

# Format code (required before commits)
cargo fmt

# Lint with clippy (required before commits)
cargo clippy

# Run tests
cargo test

# Run specific test
cargo test <test_name>
```

## Architecture Overview

The application consists of four main components:

1. **System Tray Service** (`tray.rs`)
   - Uses `ksni` crate for KDE Status Notifier Item protocol
   - Displays countdown and provides quick actions menu
   - Must handle GNOME's requirement for AppIndicator extension

2. **Timer Engine** (`timer.rs`)
   - Tokio-based async timers for micro and long breaks
   - Runs independently of UI thread
   - Manages pause/resume state and tracks next break times

3. **Settings Window** (`settings_window.rs`)
   - GTK4/libadwaita UI for configuration
   - Connects to Settings management for persistence
   - Should follow GNOME HIG design patterns

4. **Break Overlay Window** (`break_window.rs`)
   - Fullscreen semi-transparent GTK4 window
   - Must work on both X11 and Wayland (GTK4 handles this automatically)
   - Displays countdown timer and optional skip button

### Inter-Component Communication

- Use Tokio channels for timer → UI communication
- GTK4 signals for UI events → business logic
- Settings changes should propagate to Timer Engine atomically

## Configuration & Storage

Settings persist to: `~/.config/lookout/config.json`

Expected schema:
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

Use `dirs` crate for XDG-compliant config directory location.

## Critical Technical Requirements

### Display Server Support
- Single codebase for X11 and Wayland
- GTK4 handles detection at runtime - no conditional compilation needed
- Test on both display servers before major releases

### Desktop Environment Compatibility
System tray works natively on: KDE, XFCE, MATE, Cinnamon, LXQt, tiling WMs with compatible bars

**GNOME caveat**: Requires user to install AppIndicator extension. Consider:
- Detecting GNOME environment on first run
- Showing one-time notification with setup instructions
- Graceful fallback if tray isn't available

### Memory Constraints
Always profile memory usage:
- Idle in tray: 3-7 MB (strict requirement)
- Settings window open: 8-12 MB
- Break overlay active: 8-12 MB

Techniques to minimize footprint:
- Lazy load GTK windows (don't create settings window until first access)
- Dispose of break overlay window after dismissal
- Avoid caching large resources in memory

## Code Patterns

### Error Handling
- Never use `.unwrap()` in production code paths
- Use `?` operator with proper error types
- Log errors that shouldn't crash the app
- Fail gracefully for non-critical features (e.g., tray icon failure shouldn't exit app)

### Async/Await with GTK
GTK4 is NOT thread-safe. Use this pattern for timer → UI updates:
```rust
// From tokio task:
glib::MainContext::default().spawn(async move {
    // Update GTK widgets here
});
```

### Testing Strategy
- Unit tests for timer logic (can run without GTK)
- Integration tests for settings persistence
- Manual testing required for UI and tray behavior across DEs

## Dependencies

Core dependencies from Cargo.toml:
- `gtk4 = "0.9"` - UI framework
- `libadwaita = "0.7"` - Modern GNOME styling
- `tokio = { version = "1.0", features = ["full"] }` - Async runtime
- `ksni = "0.2"` - System tray protocol
- `serde` + `serde_json` - Settings serialization
- `dirs = "5.0"` - XDG directory paths

When adding dependencies, prioritize:
1. Memory efficiency
2. Active maintenance
3. Minimal dependency tree

## Known Technical Constraints

1. **Multi-monitor**: V1.0 only targets primary monitor for break overlay
2. **Idle detection**: Not implemented in MVP (breaks trigger on schedule regardless of user activity)
3. **GNOME tray**: Requires manual extension installation by user

## Resources

- GTK4 Rust: https://gtk-rs.org/gtk4-rs/stable/latest/book/
- libadwaita: https://gnome.pages.gitlab.gnome.org/libadwaita/
- GNOME HIG: https://developer.gnome.org/hig/
- ksni docs: https://docs.rs/ksni/

## Project Information

This project is developed and maintained entirely using Claude Code (AI).
Repository: https://github.com/raghulj/lookout
