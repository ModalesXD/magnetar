# ⚡ Magnetar

**Build beautiful, performant desktop widgets for Wayland using HTML, CSS, and JavaScript.**

Magnetar is a lightweight widget system for Hyprland that lets you create custom bars, panels, and overlays using web technologies. Think of it as Electron for your desktop shell, but actually fast.

---

## Why Magnetar?

**The Problem:** Most Wayland widget systems are either:

- Too complex (require learning new config languages)
- Too limited (can't create custom UIs)
- Too slow (Electron-based, heavy resource usage)
- Too rigid (hard to customize or extend)

**The Solution:** Magnetar gives you:

- ✨ **Web Technologies** - Use HTML/CSS/JS you already know
- 🚀 **Native Performance** - Built with Rust + GTK4 + WebKit
- 🎯 **Direct Integration** - Real-time access to Hyprland workspaces, windows, and events
- 🔧 **Simple CLI** - Manage widgets, inspect compositor state, create new widgets from templates
- 📦 **Zero Config** - Drop HTML files in `~/.config/magnetar/` and they just work

---

## Vision

Magnetar aims to be the **easiest way to customize your Wayland desktop**. We believe:

1. **Customization should be accessible** - If you can build a webpage, you can build a desktop widget
2. **Performance matters** - Your shell should be fast and responsive, not drain your battery
3. **Integration is key** - Widgets should feel native, not like separate apps
4. **Simplicity wins** - Good defaults, minimal configuration, just works

**Future Goals:**

- Support more Wayland compositors (Sway, River, etc.)
- Hot-reload widgets without restart
- Visual widget builder
- Plugin system for extending functionality
- Community widget marketplace

---

## Quick Start

### Installation

```bash
# Clone and install
git clone https://github.com/yourusername/magnetar.git
cd magnetar
chmod +x install.sh
./install.sh
```

That's it! The installer will:

- Build the optimized release binary
- Install to `/usr/local/bin/magnetar`
- Create config directory at `~/.config/magnetar/`
- Copy example topbar

### First Run

```bash
# Start Magnetar (loads all widgets from ~/.config/magnetar/)
magnetar
```

You should see a topbar with:

- Workspace indicators (click to switch)
- Active window title
- System info (CPU, memory)
- Date and time

---

## Creating Widgets

### 1. Use a Template

```bash
# Create a new widget from template
magnetar new my-sidebar --template sidebar

# Available templates: topbar, sidebar, overlay, notification
```

### 2. Write Your Own

Create `~/.config/magnetar/my-widget.html`:

```html
<!-- magnetar: layer=top, height=40, anchor=top|left|right -->
<!DOCTYPE html>
<html>
  <head>
    <style>
      body {
        background: rgba(30, 30, 46, 0.95);
        color: #cdd6f4;
        font-family: monospace;
        padding: 10px 20px;
      }
    </style>
  </head>
  <body>
    <h1>Hello Magnetar!</h1>
    <div id="workspace">Loading...</div>

    <script>
      // Get current workspace
      magnetar.invoke("hyprland.activeworkspace").then((ws) => {
        document.getElementById("workspace").textContent =
          `Workspace: ${ws.name}`;
      });

      // Listen to workspace changes
      magnetar.on("workspace:changed", (data) => {
        console.log("Switched to workspace:", data.id);
      });
    </script>
  </body>
</html>
```

### 3. Validate and Test

```bash
# Check if your widget is valid
magnetar validate ~/.config/magnetar/my-widget.html

# List all widgets
magnetar widget list

# Restart Magnetar to load new widget
magnetar
```

---

## Configuration

Widgets are configured via HTML comments:

```html
<!-- magnetar: layer=top, height=40, anchor=top|left|right, exclusive_zone=-1 -->
```

**Options:**

- `layer` - `background`, `bottom`, `top`, `overlay`
- `height` - Height in pixels
- `width` - Width in pixels (0 = full width)
- `anchor` - Position: `top`, `bottom`, `left`, `right` (combine with `|`)
- `margin_*` - Margins: `margin_top`, `margin_bottom`, `margin_left`, `margin_right`
- `exclusive_zone` - Reserve space (-1 = auto, 0 = no reserve, >0 = pixels)

---

## JavaScript API

### Get Compositor Data

```javascript
// Get all workspaces
const workspaces = await magnetar.invoke("hyprland.workspaces");

// Get active workspace
const active = await magnetar.invoke("hyprland.activeworkspace");

// Get active window
const window = await magnetar.invoke("hyprland.activewindow");

// Get all windows
const clients = await magnetar.invoke("hyprland.clients");

// Execute compositor command
await magnetar.invoke("hyprland.exec", "workspace 2");
```

### Listen to Events

```javascript
// Workspace changed
magnetar.on("workspace:changed", (data) => {
  console.log("New workspace:", data.id);
});

// Active window changed
magnetar.on("activewindow:changed", (data) => {
  console.log("Window:", data.title, data.class);
});
```

### Control Your Widget

```javascript
// Resize widget
await magnetar.invoke("window.resize", { width: 400, height: 300 });

// Hide widget
await magnetar.invoke("window.hide");

// Show widget
await magnetar.invoke("window.show");
```

### Communicate Between Widgets

```javascript
// Send event to all widgets
await magnetar.invoke("broadcast", {
  event: "my-custom-event",
  data: { foo: "bar" },
});

// Listen in other widgets
magnetar.on("my-custom-event", (data) => {
  console.log("Received:", data);
});
```

---

## CLI Commands

### Widget Management

```bash
# List all widgets
magnetar widget list

# Inspect widget configuration
magnetar widget inspect ~/.config/magnetar/topbar.html

# Create new widget from template
magnetar new my-widget --template topbar
```

### Compositor Control

```bash
# Show compositor info
magnetar compositor info

# List workspaces
magnetar compositor workspaces

# List windows
magnetar compositor clients

# Show active workspace
magnetar compositor active

# Execute command
magnetar compositor exec workspace 3
```

### System Info

```bash
# Show system information
magnetar info

# Detailed info
magnetar info -v
```

### Validation

```bash
# Validate widget configuration
magnetar validate ~/.config/magnetar/my-widget.html
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│           Magnetar Core (Rust)          │
│  ┌─────────────────────────────────┐   │
│  │   GTK4 + Layer Shell (Wayland)  │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │   WebKit6 (HTML/CSS/JS Engine)  │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │   IPC Bridge (Rust ↔ JS)        │   │
│  └─────────────────────────────────┘   │
│  ┌─────────────────────────────────┐   │
│  │   Compositor Integration        │   │
│  │   (Hyprland via hyprctl)        │   │
│  └─────────────────────────────────┘   │
└─────────────────────────────────────────┘
           ↕ Unix Sockets
┌─────────────────────────────────────────┐
│         Hyprland Compositor             │
└─────────────────────────────────────────┘
```

**Key Components:**

- **Window Manager** (`src/window/`) - GTK4 layer-shell windows
- **IPC System** (`src/ipc/`) - Bidirectional Rust ↔ JavaScript communication
- **Compositor Integration** (`src/compositor/`) - Hyprland event subscription and control
- **Config Loader** (`src/config/`) - Parse and load widget configurations
- **CLI** (`src/cli/`) - Command-line interface for management
- **Runtime** (`src/runtime.rs`) - Shared Tokio async runtime (optimized, 2 threads)

---

## Performance

Magnetar is designed for performance:

- **Optimized Runtime** - Shared Tokio runtime with 2 worker threads
- **Efficient IPC** - Async channels, no blocking operations
- **Smart Rendering** - WebKit GPU acceleration
- **Low Memory** - ~30-50MB per widget (vs 200-300MB for Electron)
- **Fast Startup** - <100ms to first render
- **Release Optimizations** - LTO, strip, single codegen unit

**Benchmarks** (on typical system):

- Startup time: ~80ms
- Memory usage: ~40MB (with topbar)
- CPU usage (idle): <0.5%
- Event latency: <5ms

---

## Requirements

- **OS:** Linux with Wayland
- **Compositor:** Hyprland (more coming soon)
- **Dependencies:**
  - GTK4
  - WebKit2GTK 6.0
  - gtk4-layer-shell

### Install Dependencies

**Arch Linux:**

```bash
sudo pacman -S gtk4 webkit2gtk-6.0 gtk4-layer-shell
```

**Ubuntu/Debian:**

```bash
sudo apt install libgtk-4-dev libwebkit2gtk-6.0-dev libgtk-layer-shell-dev
```

**Fedora:**

```bash
sudo dnf install gtk4-devel webkit2gtk4.1-devel gtk4-layer-shell-devel
```

---

## Development

```bash
# Build
cargo build

# Run in debug mode
cargo run

# Run with debug logging
RUST_LOG=magnetar=debug cargo run

# Run CLI commands
cargo run -- widget list
cargo run -- compositor workspaces

# Format code
cargo fmt

# Lint
cargo clippy

# Build optimized release
cargo build --release
```

---

## Troubleshooting

### Widget not showing?

```bash
# Check if Magnetar is running
ps aux | grep magnetar

# Check widget configuration
magnetar validate ~/.config/magnetar/your-widget.html

# Check logs
RUST_LOG=magnetar=debug magnetar
```

### Compositor not detected?

```bash
# Verify Hyprland is running
echo $HYPRLAND_INSTANCE_SIGNATURE

# Check compositor info
magnetar compositor info
```

### JavaScript errors?

Open the WebKit inspector (if enabled in debug builds) or check console output:

```bash
RUST_LOG=magnetar=debug magnetar 2>&1 | grep -i javascript
```

---

## Contributing

Contributions are welcome! Please:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing`)
3. Commit your changes (`git commit -am 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing`)
5. Open a Pull Request

**Areas we need help:**

- Support for more compositors (Sway, River)
- Widget templates and examples
- Documentation and tutorials
- Performance optimizations
- Bug fixes

---

## License

MIT License - see LICENSE file for details

---

## Credits

Built with:

- [Rust](https://www.rust-lang.org/) - Systems programming language
- [GTK4](https://www.gtk.org/) - UI toolkit
- [WebKit](https://webkit.org/) - Web rendering engine
- [Tokio](https://tokio.rs/) - Async runtime
- [Hyprland](https://hyprland.org/) - Wayland compositor

Inspired by:

- [Ags](https://github.com/Aylur/ags) - Awesome GTK Shell
- [Eww](https://github.com/elkowar/eww) - ElKowar's Wacky Widgets
- [Waybar](https://github.com/Alexays/Waybar) - Wayland bar

---

## Support

- **Issues:** [GitHub Issues](https://github.com/yourusername/magnetar/issues)
- **Discussions:** [GitHub Discussions](https://github.com/yourusername/magnetar/discussions)
- **Discord:** Coming soon

---

**Made with ⚡ by the Magnetar team**
