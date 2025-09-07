# CommonUI

[![License: MIT OR Apache-2.0](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](https://github.com/commonui/commonui)

CommonUI is a **fully-native, high-performance, reactive GUI toolkit** for Rust, designed specifically for games and performance-critical applications. Built on modern foundations with thread-safe reactivity and 60FPS rendering capabilities. Another key differentiator is CommonUI's first class support for wgpu-based apps, so you can build your own wgpu pipeline and render safely to the same surface as this GUI kit.

## ✨ Features

- **🚀 High Performance**: Game-ready 60FPS rendering with Vello GPU acceleration
- **🔄 Thread-Safe Reactivity**: Signals, computed values, and effects with frame synchronization
- **📐 Flexible Layout**: Powered by Taffy layout engine with flexbox and grid support
- **🎨 Modern Rendering**: GPU-accelerated vector graphics with scene caching
- **⚡ Async-Ready**: Built on Tokio for responsive UI updates

## 🏗️ Architecture

CommonUI is built as a modular workspace with specialized crates:

- **`gui-core`**: Core widget system and application framework
- **`gui-reactive`**: Thread-safe signals and reactive primitives
- **`gui-layout`**: Layout engine integration with invalidation system
- **`gui-render`**: Vello-based rendering backend with optimization
- **`commonui-examples`**: Example applications and demos

## 🚀 Quick Start

```rust
use gui_core::{App, Element};
use gui_core::widgets::*;
use vello::peniko::Color;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple UI
    let hello_text = text("Hello, CommonUI!")
        .with_font_size(24.0)
        .with_color(Color::rgba8(250, 250, 200, 255));
    
    let button = button("Click Me!")
        .with_size(120.0, 40.0);
    
    // Arrange in a column layout
    let layout = column()
        .with_size(300.0, 200.0)
        .with_main_axis_alignment(MainAxisAlignment::Center)
        .with_gap(20.0)
        .with_child(Element::new_widget(Box::new(hello_text)))
        .with_child(Element::new_widget(Box::new(button)));
    
    // Run the application
    let app = App::new().with_root(layout.into_container_element())?;
    app.run()
}
```

## 📦 Installation

Add CommonUI to your `Cargo.toml`:

```toml
[dependencies]
gui-core = { path = "gui-core" }
vello = "0.1"
```

## 🏃‍♀️ Running Examples

```bash
# Run the hello world example
cargo run --bin commonui-examples

# Check all crates
cargo check --workspace
```

## 🎯 Core Technologies

| Component | Technology | Purpose |
|-----------|------------|---------|
| **Rendering** | [Vello](https://github.com/linebender/vello) + wgpu | GPU-accelerated 2D graphics |
| **Layout** | [Taffy](https://github.com/DioxusLabs/taffy) | CSS-like layout engine |
| **Text** | [cosmic-text](https://github.com/pop-os/cosmic-text) | Advanced text rendering |
| **Windowing** | [winit](https://github.com/rust-windowing/winit) | Cross-platform windows |
| **Async** | [Tokio](https://tokio.rs/) | Async runtime for reactivity |

## 🎨 Widget System

CommonUI provides a growing collection of widgets:

- **Layout**: Container, Row, Column, Stack
- **Text**: Text with styling and cosmic-text integration  
- **Interactive**: Button, Input, Slider
- **Advanced**: Custom widgets with reactive state

## 🔄 Reactive System

Built-in reactive primitives for dynamic UIs:

```rust
// Signals for mutable state
let count = Signal::new(0);

// Computed values that automatically update
let doubled = Computed::new(move || count.get() * 2);

// Effects for side effects
Effect::new(move || {
    println!("Count is now: {}", count.get());
});
```

## 🛣️ Roadmap

- [x] Core reactive system with signals and effects
- [x] Basic widget foundation (Text, Button, Container)
- [x] Layout system with Taffy integration
- [x] Vello rendering backend
- [x] Hello World example
- [ ] Advanced input handling and focus management
- [ ] Accessibility support
- [ ] Complex layout examples
- [ ] Performance benchmarks and optimization
- [ ] Comprehensive documentation

## 🤝 Contributing

CommonUI is actively developed with Claude Code assistance. Contributions are welcome!

## 📄 License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## 🙏 Inspiration

CommonUI draws inspiration from [Floem](https://github.com/lapce/floem) and modern reactive UI frameworks, adapted for high-performance Rust applications.
