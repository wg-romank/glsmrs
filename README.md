# WebGL State Machine in Rust

TODO: description
TODO: main concepts (Program, GlState)

## Usage example

Import crate

```rust
use glsmrs as gl;
```

Create a program description

```rust
gl::Program::new(
    ctx,
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
let mut state = gl::GlState::new(&ctx, viewport);

state
    .vertex_buffer("position", vb)?
    .vertex_buffer("uv", uv)?
    .texture("tex", Some(tex_byts), size, size)?
    .texture("buf", None, size, size)?
    .element_buffer(eb)?;
```

Use program & state together

```rust
fn animation_step(program: &gl::Program, state: &gl::GlState, time: u32) -> Result<(), String> {
    let uniforms: HashMap<_, _> = vec![
        ("tex", gl::UniformData::Texture("tex")),
        ("time", gl::UniformData::Scalar(time as f32)),
    ].into_iter().collect();

    state.run_mut(&program, &uniforms, "buf")?;

    let uniforms2: HashMap<_, _> = vec![
        ("tex", gl::UniformData::Texture("buf")),
        ("time", gl::UniformData::Scalar(time as f32)),
    ].into_iter().collect();

    state.run(&program, &uniforms2)?;

    Ok(())
}
```