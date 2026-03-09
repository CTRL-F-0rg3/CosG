# CosG

[![crates.io](https://img.shields.io/crates/v/cosg.svg)](https://crates.io/crates/cosg)
[![license](https://img.shields.io/crates/l/cosg.svg)](LICENSE)

A wgpu-based UI library for Rust with a dark violet aesthetic. Built for use in CosinusOS and beyond.

## Features

- **wgpu renderer** — GPU-accelerated rect rendering with SDF rounded corners
- **Widget system** — `Button`, `Label`, `Container`, `Grid` out of the box
- **ESC system** — optional extra style controls per widget (`border_radius`, `shadow`, `opacity`, `border`)
- **Theme system** — violet dark theme by default, fully customizable

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
cosg = "0.1.0"
```

Basic example:

```rust
use cosg::*;

fn main() {
    App::new(
        AppConfig {
            title:  "My App".into(),
            width:  800,
            height: 600,
            theme:  Theme::violet_dark(),
        },
        || {
            let btn = Button::new()
                .label("Click me")
                .esc(Esc::new().border_radius(12.0))
                .on_press(|| println!("pressed!"));

            Box::new(
                Container::new()
                    .padding(24.0)
                    .add(Label::new("Hello CosG"))
                    .add(btn),
            )
        },
    )
    .run();
}
```

## Architecture

### Widget Trait

Every UI element implements `Widget`:

```rust
pub trait Widget {
    fn layout(&mut self, bounds: Rect, theme: &Theme);
    fn draw(&self, theme: &Theme) -> Vec<DrawRect>;
    fn handle_event(&mut self, event: &WidgetEvent);
    fn rect(&self) -> Rect;
}
```

Layout is driven top-down — the parent calls `layout()` on each child with the bounds it allocates.

### ESC System

ESC (Extra Style Controls) is a separate layer of optional visual properties that sits on top of the theme. The theme controls colors and font sizes globally; ESC lets you override specific visual properties per widget:

```rust
let btn = Button::new()
    .esc(Esc::new()
        .border_radius(8.0)
        .shadow(4.0)
        .opacity(0.9)
        .border_width(1.0)
    );
```

### Theme

```rust
let theme = Theme::violet_dark(); // default
```

You can construct a custom `Theme` by filling in the struct fields directly — colors are `[f32; 4]` RGBA.

### Layout Primitives

**Container** — vertical stack with padding:
```rust
Container::new()
    .padding(16.0)
    .add(widget_a)
    .add(widget_b)
```

**Grid** — place widgets at (col, row):
```rust
Grid::new(2, 2)
    .gap(8.0)
    .place(label,  0, 0)
    .place(button, 1, 0)
```

## Roadmap

- [ ] Text rendering via `glyphon`
- [ ] `TextInput` widget
- [ ] Horizontal layout in `Container`
- [ ] Focus management (keyboard, Tab)
- [ ] Hover/press animations
- [ ] `colspan` / `rowspan` in Grid
- [ ] Scrollable containers
- [ ] Shadow rendering in shader

## License

MIT