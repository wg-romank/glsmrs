use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext};

use std::collections::HashMap;

use std::panic;
use console_error_panic_hook;

mod gl;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn alter_start(ctx: &WebGlRenderingContext) -> Result<(), String> {
    let program = gl::Program::new(
        ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/dummy.frag"),
        vec![
            gl::UniformDescription { name: "tex", location: None, t: gl::UniformType::Sampler2D }
        ],
        vec![
            gl::AttributeDescription { name: "position", location: None, t: gl::AttributeType::Vector(2) },
            gl::AttributeDescription { name: "uv", location: None, t: gl::AttributeType::Vector(2) }
        ]
    )?;

    let mut state = gl::GlState::new();

    let vertices: [f32; 8] = [
        -0.5, -0.5,
        0.5, -0.5,
        0.5, 0.5,
        -0.5, 0.5
    ];
    let uvs: [f32; 8] = [
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
        0.0, 1.0
    ];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    let vb: Vec<u8> = vertices.iter().flat_map(|v| v.to_ne_bytes().to_vec()).collect();
    let uv: Vec<u8> = uvs.iter().flat_map(|u| u.to_ne_bytes().to_vec()).collect();
    let eb: Vec<u8> = indices.iter().flat_map(|e| e.to_ne_bytes().to_vec()).collect();

    // let tex: Vec<f32> = (0..(32 * 32)).map(|idx: u32| (idx as f32)).collect();
    let mut tex_byts: Vec<u8> = vec![];
    let size = 32;
    for col in 0..size {
        for row in 0..size {
            // rgba is 32 bit, thus here we want to encode our data using u32
            let red_bytes = ((row * size + col) as u32).to_ne_bytes().to_vec();
            // times 256 to shift to next channel
            let gree_bytes = ((row * size + col) * 256 as u32).to_ne_bytes().to_vec();
            for b in red_bytes {
                tex_byts.push(b);
            }
        }
    }

    state
        .vertex_buffer(ctx, "position", vb.as_slice())?
        .vertex_buffer(ctx, "uv", uv.as_slice())?
        // .texturef(ctx, "tex", tex.as_slice(), 32, 32)?
        .texture(ctx, "tex", tex_byts.as_slice(), size, size)?
        .element_buffer(ctx, eb.as_slice())?;


    let mut uniforms = HashMap::new();
    uniforms.insert("tex", gl::UniformData::Texture("tex"));

    state.run(ctx, &program, uniforms)?;

    Ok(())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .unwrap()
        .dyn_into::<WebGlRenderingContext>()?;

    // context.get_extension("OES_texture_float")?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    alter_start(&context)?;

    Ok(())
}