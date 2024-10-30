# Cleave

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-stable-orange.svg)

Cleave is a lightweight, keyboard-driven screenshot tool written in Rust that enables rapid screen capture and selection. It provides a clean, minimalist interface for capturing portions of your screen with pixel-perfect accuracy.

## ✨ Key Features

- **Full-screen Selection**: Click and drag to select any portion of your screen
- **Keyboard Navigation**: Quick commands for capture and exit
- **Instant Clipboard Integration**: Selected areas automatically copy to clipboard
- **Visual Feedback**: Real-time visual indicators for selection area
- **Multi-monitor Support**: Works with primary display
- **Zero Configuration**: Works out of the box with sensible defaults

## 🚀 Quick Start

1. Launch Cleave
2. Click and drag to select an area
3. Press `Space` to capture and copy to clipboard
4. Press `Esc` to cancel and exit

## 📦 Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/cleave
cd cleave

# Build and install
cargo install --path .
```

## 🎮 Usage

### Basic Controls

| Key/Action | Description |
|------------|-------------|
| Left Click + Drag | Select area |
| Space | Capture selection to clipboard |
| Escape | Exit application |
| Right Click | Cancel current selection |

### Command Line

```bash
# Launch Cleave
cleave
```

## ⚙️ Configuration

Cleave is designed to work without configuration. Current default behaviors:

- Captures from primary monitor
- Saves directly to clipboard
- Shows red border for selection area
- Uses 50% opacity for selection overlay

## 🔧 Technical Details

Cleave is built with:
- `winit` for window management
- `pixels` for efficient rendering
- `arboard` for clipboard operations
- `xcap` for screen capture
- `image` for image processing

## 🤝 Contributing

Contributions are welcome! Here's how you can help:

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

### Development Setup

```bash
# Install development dependencies
cargo build

# Run tests
cargo test

# Run with debug output
cargo run
```

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🐛 Known Issues

- Currently only supports the primary monitor
- Selection border is fixed to red color
- No support for window-specific capture

## 🎯 Roadmap

- [ ] Multi-monitor support
- [ ] Configurable border colors
- [ ] Save to file option
- [ ] Hotkey configuration
- [ ] Window detection and snapping

## 💡 Tips

- For pixel-perfect selection, use slower mouse movements
- Right-click to cancel if you make a mistake
- The selection border is one pixel wide for precision

