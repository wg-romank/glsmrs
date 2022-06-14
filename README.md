# WebGL State Machine in Rust

[![Package][package-img]][package-url] ![Build](https://github.com/wg-romank/glsmrs/workflows/Build/badge.svg?branch=master) ![Publish](https://github.com/wg-romank/glsmrs/workflows/Publish/badge.svg)

An opinionated wrapper for low-level WebGL API with intention to provide a bit of explicit state management with reasonable defaults.
Primary goals for this library is to support WebGL 1.0 so it is built on top of raw bindings from `web-sys` crate.

## Key concepts

- **Program** - GL program description including vertex shader, fragment shader, attributes and uniforms. Program takes care of compiling shaders, getting attributes / uniforms locations and finally disposing resources once it goes out of scope.
- **Mesh** - structure that holds references to data uploaded to GPU, takes care of disposing array / element buffers once it goes out of scope.
- **Framebuffer** - render target, has depth and color slot, can also be initialized as empty then rendering would go to the screen.
- **Pipeline** - a primitive for drawing stuff to screen, sole purpose of which is to set GL context configuration and provide `shade` method for drawing.

## Usage example

Get it from crates.io

```toml
[dependencies]
# ...
glsmrs = "0.2.0"
```

Import crate

```rust
use glsmrs as gl;
```

Create context, `Ctx` is a wrapper that is using `Rc` internally so you can clone it and pass around without worrying too much about lifetimes.

```rust
let ctx = gl::util::get_ctx("canvas-name", "webgl")?;
```

Create some mesh, like an RGB triangle

```rust
let vertices = vec![[0.5, -0.5], [0.0, 0.5], [-0.5, -0.5]];
let colors = vec![[1., 0., 0.], [0., 1., 0.], [0., 0., 1.]];
let indices = [0, 1, 2];

let triangle = gl::mesh::Mesh::new(&ctx, &indices)?
    .with_attribute::<gl::attributes::AttributeVector2>("position", &vertices)?
    .with_attrubute::<gl::attributes::AttributeVector3>("color", &colors)?;
```

Define render target

```rust
let viewport = gl::texture::Viewport::new(720, 480);
let displayfb = gl::texture::EmptyFramebuffer::new(&ctx, viewport);
```

Create a program description

```rust
let program = gl::Program::new(
    &ctx,
    include_str!("../shaders/dummy.vert"),
    include_str!("../shaders/dummy.frag"),
)?;
```

Run program on state supplying necessary inputs

```rust
let uniforms: HashMap<_, _> = vec![
    ("time", gl::UniformData::Scalar(time as f32)),
].into_iter().collect();

let pipeline = gl::Pipeline::new(&ctx);

pipeline.shade(
    &program,
    uniforms,
    vec![&mut triangle],
    &mut displayfb
)?;
```

For example project using this library check out [ https://github.com/wg-romank/wasm-game-of-life ] a tweaked version of original WASM tutorial that runs entierly on GPU.

[package-img]: https://img.shields.io/crates/v/glsmrs
[package-url]: https://crates.io/crates/glsmrs
