# Rust 3D Grapher

A mathematical function grapher built with Rust and eframe/egui.

## Features

- Interactive 3D visualization of mathematical functions
- Real-time expression parsing and graphing
- Rotation and zoom controls for 3D view
- Supports all standard mathematical functions and operators
- Color-coded visualization to represent depth
- Resizable panels with the graph automatically taking most of the screen space

## Usage

1. Enter a mathematical expression in the bottom panel text field
2. Press Enter or click the "Graph" button to visualize the function
3. Use the rotation and zoom controls to adjust the 3D view
4. Try the example expressions by clicking on them in the "Example Expressions" dropdown

## Supported Functions

- Basic arithmetic: +, -, *, /, ^, %
- Trigonometric: sin, cos, tan
- Inverse trigonometric: asin, acos, atan, atan2
- Hyperbolic: sinh, cosh, tanh
- Other: exp, ln, log, log10, abs, sqrt

## Building and Running

Make sure you have Rust and Cargo installed, then:

```bash
cargo run --release
```

## Dependencies

- egui - Immediate mode GUI library for Rust
- eframe - Cross-platform application framework based on egui
- egui_plot - Plotting functionality for egui
- meval - Mathematical expression parser and evaluator
- glam - 3D mathematics library for Rust

## License

MIT License
