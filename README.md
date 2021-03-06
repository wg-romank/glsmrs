# WebGL State Machine in Rust

[![Package][package-img]][package-url] ![Build](https://github.com/wg-romank/glsmrs/workflows/Build/badge.svg?branch=master) ![Publish](https://github.com/wg-romank/glsmrs/workflows/Publish/badge.svg)

An opinionated wrapper for low-level WebGL API with intention to provide a bit of explicit state management with reasonable defaults.
Primary goals for this library is to support WebGL 1.0 so it is built on top of raw bindings from `web-sys` crate.

## Key concepts

- **Program** - GL program description including vertex shader, fragment shader, attributes and uniforms. Program takes care of compiling shaders, getting attributes / uniforms locations and finally disposing resources once it goes out of scope.
- **GlState** - container holding references to data that was created / sent to GPU. GlState's main purpose to keep track of various bits in the state machine (array buffer, frame buffer, active texture, etc.) and adjust accordingly. It also implements housekeeping for unused resources.

## Usage example

Get it from crates.io

```toml
[dependencies]
# ...
glsmrs = "0.1.1"
```

Import crate

```rust
use glsmrs as gl;
```

Create a program description

```rust
gl::Program::new(
    &ctx,
    include_str!("../shaders/dummy.vert"),
    include_str!("../shaders/dummy.frag"),
    vec![
        gl::UniformDescription::new("tex", gl::UniformType::Sampler2D),
        gl::UniformDescription::new("time", gl::UniformType::Float),
    ],
    vec![
        gl::AttributeDescription::new("position", gl::AttributeType::Vector2),
        gl::AttributeDescription::new("uv", gl::AttributeType::Vector2),
    ]
)
```

Initialize & prepare state of GLSM

```rust
let state = gl::GlState::new(&ctx, viewport)
    .vertex_buffer("position", vb)?
    .vertex_buffer("uv", uv)?
    .texture("tex", Some(tex_byts), size, size)?
    .element_buffer(eb)?;
```

Run program on state supplying necessary inputs

```rust
let uniforms: HashMap<_, _> = vec![
    ("tex", gl::UniformData::Texture("tex")),
    ("time", gl::UniformData::Scalar(time as f32)),
].into_iter().collect();

state.run(&program, &uniforms)?;
```

For example project using this library check out [ https://github.com/wg-romank/wasm-game-of-life ] a tweaked version of original WASM tutorial that runs entierly on GPU.

[package-img]: https://img.shields.io/crates/v/glsmrs
[package-url]: https://crates.io/crates/glsmrs
