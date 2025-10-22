# Contributing to Lookout

## Development Setup

### Prerequisites

1. **Install Rust** (if not already installed):
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

2. **Install GTK4 development libraries**:

**Ubuntu/Debian:**
```bash
sudo apt install libgtk-4-dev libadwaita-1-dev build-essential
```

**Fedora:**
```bash
sudo dnf install gtk4-devel libadwaita-devel gcc
```

**Arch Linux:**
```bash
sudo pacman -S gtk4 libadwaita base-devel
```

### Building the Project

```bash
# Clone the repository
git clone <repository-url>
cd lookout

# Build in debug mode
make build
# or
cargo build

# Build in release mode
make release
# or
cargo build --release
```

## Code Standards

### Formatting

We use `rustfmt` for consistent code formatting. Configuration is in `rustfmt.toml`.

```bash
# Format code
make fmt
# or
cargo fmt

# Check formatting without modifying files
make fmt-check
# or
cargo fmt -- --check
```

### Linting

We use `clippy` for linting with strict rules. Configuration is in `clippy.toml`.

```bash
# Run clippy
make lint
# or
cargo clippy --all-targets --all-features
```

**Important**: The project has strict linting rules:
- No `unwrap()` or `expect()` in production code (use proper error handling)
- All clippy warnings treated as errors
- Cognitive complexity threshold: 15

### Testing

```bash
# Run all tests
make test
# or
cargo test

# Run specific test
cargo test <test_name>

# Run tests with output
cargo test -- --nocapture
```

### Complete Development Check

Run all checks before committing:

```bash
make check
# This runs: format check, lint, and tests
```

## Development Workflow

1. **Create a feature branch**:
```bash
git checkout -b feature/your-feature-name
```

2. **Make your changes**

3. **Run checks**:
```bash
make dev
# This runs: format, lint, test, and build
```

4. **Commit your changes**:
```bash
git add .
git commit -m "Description of changes"
```

5. **Push and create PR**:
```bash
git push origin feature/your-feature-name
```

## Project Structure

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
├── Cargo.toml               # Dependencies and build config
├── rustfmt.toml             # Formatting rules
├── clippy.toml              # Linting rules
└── Makefile                 # Build shortcuts
```

## Coding Guidelines

### Error Handling

Always use proper error handling:

```rust
// ❌ Bad
let value = something.unwrap();

// ✅ Good
let value = something.map_err(|e| format!("Failed: {}", e))?;
```

### Async Code with GTK

GTK is not thread-safe. Use this pattern for updates from async tasks:

```rust
// From tokio task:
glib::MainContext::default().spawn(async move {
    // Update GTK widgets here
});
```

### Memory Efficiency

Always consider memory usage:
- Lazy load GTK windows
- Dispose of unused windows
- Avoid caching large resources
- Profile with `valgrind` or `/usr/bin/time -v`

### Documentation

Document public APIs:

```rust
/// Brief description of what this does
///
/// # Arguments
/// * `param` - Description of parameter
///
/// # Errors
/// Returns error if...
pub fn my_function(param: Type) -> Result<(), Error> {
    // implementation
}
```

## Testing Desktop Environments

Test on multiple DEs before major releases:
- KDE Plasma
- GNOME (with AppIndicator extension)
- XFCE
- Test both X11 and Wayland

## Questions?

Open an issue or discussion on GitHub.
