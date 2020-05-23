use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlProgram, WebGlRenderingContext, WebGlShader};

use std::collections::HashMap;

use std::panic;
use console_error_panic_hook;

mod gl;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

pub fn alter_start(ctx: &WebGlRenderingContext) -> Result<(), &'static str> {
    let program = gl::Program::new(
        ctx,
        include_str!("../shaders/dummy.vert"),
        include_str!("../shaders/dummy.frag"),
        vec![],
        vec![
            gl::AttributeDescription { name: "position", location: None, t: gl::AttributeType::Vector(3) }
        ]
    )?;

    let mut state = gl::GlState::new();

    let vertices: [f32; 16] = [
        // Position         UV
        -1., -1.,         0.0, 0.0,
        1., -1.,          1.0, 0.0,
        1., 1.,           1.0, 1.0,
        -1., 1.,          0.0, 1.0,
    ];
    let indices: [u8; 6] = [0, 1, 2, 2, 3, 0];

    let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];
    let vb: Vec<u8> = vertices.into_iter().flat_map(|v| v.to_ne_bytes().to_vec()).collect();
    let indices: [u16; 3] = [0, 1, 2];
    let eb: Vec<u8> = indices.into_iter().flat_map(|e| e.to_ne_bytes().to_vec()).collect();

    state
        .vertex_buffer(ctx, "position", vb.as_slice()).ok_or("Failed to create vertex buffer")?
        .element_buffer(ctx, eb.as_slice()).ok_or("Failed to create element buffer")?;


    state.run(ctx, &program, HashMap::new()).ok_or("Failed to run program")?;

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


    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    alter_start(&context)?;

    Ok(())
}