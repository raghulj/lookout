# Lookout

A lightweight break reminder app for Linux that helps reduce eye strain and digital fatigue through micro breaks and long breaks.

Built with Rust, GTK4, and libadwaita. Runs in the system tray with <10MB memory footprint.

---

## 🚀 Quick Install

Install Lookout with a single command:

```bash
curl -fsSL https://raw.githubusercontent.com/raghulj/lookout/main/install.sh | bash
```

**That's it!** The installer will:
- ✅ Detect your system automatically
- ✅ Download the latest release
- ✅ Install to `~/.local/bin` (no sudo required)
- ✅ Set up desktop entry
- ✅ Check for required dependencies

### Requirements

Lookout requires GTK4 and libadwaita:

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-1 libadwaita-1-0
```

**Fedora:**
```bash
sudo dnf install gtk4 libadwaita
```

**Arch Linux:**
```bash
sudo pacman -S gtk4 libadwaita
```

---

## 📖 Usage

**Start the app:**
```bash
lookout
```

Or find **Lookout** in your application launcher.

**Enable autostart:**
```bash
lookout --enable-autostart
```

**Disable autostart:**
```bash
lookout --disable-autostart
```

**Check version:**
```bash
lookout --version
```

**Get help:**
```bash
lookout --help
```

---

## 🔄 Updating

To update to the latest version, run the install command again:

```bash
curl -fsSL https://raw.githubusercontent.com/raghulj/lookout/main/install.sh | bash
```

---

## 🗑️ Uninstalling

```bash
curl -fsSL https://raw.githubusercontent.com/raghulj/lookout/main/uninstall.sh | bash
```

---

## ✨ Features

### Two Break Types
- **Micro breaks** - 20-second breaks every 20 minutes (customizable)
- **Long breaks** - 5-minute breaks every 60 minutes (customizable)

### System Tray Integration
- Runs quietly in the background
- Shows next break time
- Right-click menu for quick actions
- Pause/resume breaks anytime

### Break Overlays
- Fullscreen semi-transparent overlay
- Countdown timer display
- Skip button (after minimum duration)
- Works on both X11 and Wayland

### Settings Window
- Configure break intervals and durations
- Start/pause/resume functionality
- Auto-start on login option
- Remembers your preferences

### Design Highlights
- Minimal, clean UI following GNOME HIG
- Low memory footprint (<10MB idle)
- Native look and feel
- Smooth animations

---

## 🖥️ Desktop Environment Support

### Fully Supported (System Tray Works Out of the Box)
- KDE Plasma ✅
- XFCE ✅
- MATE ✅
- Cinnamon ✅
- LXQt ✅
- Tiling WMs with compatible bars (i3, Sway with waybar, etc.) ✅

### GNOME Support
Requires [AppIndicator extension](https://extensions.gnome.org/extension/615/appindicator-support/)

---

## 🏗️ Technical Stack

- **Language**: Rust
- **UI Framework**: GTK4
- **Styling**: libadwaita
- **Async Runtime**: tokio
- **System Tray**: ksni (KDE Status Notifier Item)

### Display Server Support
- **X11** ✅
- **Wayland** ✅

GTK4 automatically handles both - single codebase, works everywhere.

---

## 📦 Manual Installation

If you prefer to install manually:

1. Download the binary for your architecture from [releases](https://github.com/raghulj/lookout/releases/latest)
2. Make it executable:
   ```bash
   chmod +x lookout-x86_64-unknown-linux-gnu
   ```
3. Move to your PATH:
   ```bash
   mkdir -p ~/.local/bin
   mv lookout-x86_64-unknown-linux-gnu ~/.local/bin/lookout
   ```
4. Add to PATH (if not already):
   ```bash
   echo 'export PATH="$HOME/.local/bin:$PATH"' >> ~/.bashrc
   source ~/.bashrc
   ```

### Supported Architectures
- **x86_64** (Intel/AMD 64-bit) - Available now
- **aarch64** (ARM 64-bit) - Planned
- **armv7** (ARM 32-bit) - Planned

---

## ⚙️ Configuration

Settings are stored in: `~/.config/lookout/config.json`

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

You can edit this file directly or use the settings window in the app.

---

## 🛠️ Building from Source

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt install libgtk-4-dev libadwaita-1-dev libdbus-1-dev build-essential pkg-config

# Fedora
sudo dnf install gtk4-devel libadwaita-devel dbus-devel gcc pkg-config

# Arch Linux
sudo pacman -S gtk4 libadwaita dbus base-devel
```

### Build

```bash
# Clone the repository
git clone https://github.com/raghulj/lookout.git
cd lookout

# Build release binary
cargo build --release

# Binary will be at: target/release/lookout
./target/release/lookout
```

---

## 🎯 Project Goals

- Simple, efficient break reminder that just works
- Native Linux application (not Electron!)
- Minimal memory footprint (<10MB)
- Works across all major desktop environments
- Respects user privacy (no telemetry, no network calls)

---

## 📝 Known Limitations

1. **GNOME System Tray**: Requires AppIndicator extension
2. **Idle Detection**: Not implemented yet (breaks trigger on schedule regardless of user activity)
3. **Multi-Monitor**: Break overlay shows on primary monitor only (v1.0)

---

## 🚧 Future Enhancements

- [ ] Idle time detection
- [ ] Multi-monitor support
- [ ] Break exercise suggestions/animations
- [ ] Statistics and usage insights
- [ ] Sound notifications
- [ ] Theme customization
- [ ] Pomodoro mode
- [ ] Flatpak package
- [ ] AUR package (Arch User Repository)

---

## 🤝 Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.

### Development Setup

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

### Reporting Issues

Found a bug or have a feature request? [Open an issue](https://github.com/raghulj/lookout/issues/new)

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## 🙏 Acknowledgments

- Built entirely using [Claude Code](https://claude.com/claude-code) (AI pair programmer)
- Inspired by the need for simple, native break reminder apps on Linux
- Thanks to the GTK and Rust communities for excellent tooling

---

## 📞 Support

- **Issues**: [GitHub Issues](https://github.com/raghulj/lookout/issues)
- **Discussions**: [GitHub Discussions](https://github.com/raghulj/lookout/discussions)

---

**Star ⭐ this repository if you find it useful!**
