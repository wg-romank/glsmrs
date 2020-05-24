use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{WebGlRenderingContext};

use std::collections::HashMap;

use std::panic;
use std::cell::RefCell;
use std::rc::Rc;

use console_error_panic_hook;

mod gl;

macro_rules! log {
    ( $( $t:tt )* ) => {
        web_sys::console::log_1(&format!( $( $t )* ).into());
    }
}

fn display_program(ctx: &WebGlRenderingContext) -> Result<gl::Program, String> {
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
}

fn setup_state(ctx: &WebGlRenderingContext, viewport: gl::Viewport, vertices: [f32; 8], uvs: [f32; 8], indices: [u16; 6]) -> Result<gl::GlState, String> {
    let mut state = gl::GlState::new(viewport);

    let vb: Vec<u8> = vertices.iter().flat_map(|v| v.to_ne_bytes().to_vec()).collect();
    let uv: Vec<u8> = uvs.iter().flat_map(|u| u.to_ne_bytes().to_vec()).collect();
    let eb: Vec<u8> = indices.iter().flat_map(|e| e.to_ne_bytes().to_vec()).collect();

    let mut tex_byts: Vec<u8> = vec![];
    let size = 256;
    for col in 0..size {
        for row in 0..size {
            // rgba is 32 bit, thus here we want to encode our data using u32
            let red_bytes = ((row * size + col) as u32).to_ne_bytes().to_vec();
            // times 256 to shift to next channel
            // let green_bytes = ((row * size + col) * 256 as u32).to_ne_bytes().to_vec();
            //    col
            //
            //     |
            //  
            //  (0, 0)   ->  row
            for b in red_bytes {
                tex_byts.push(b);
            }
        }
    }

    state
        .vertex_buffer(ctx, "position", vb.as_slice())?
        .vertex_buffer(ctx, "uv", uv.as_slice())?
        .texture(ctx, "tex", Some(tex_byts.as_slice()), size, size)?
        .texture(&ctx, "buf", None, 256, 256)?
        .element_buffer(ctx, eb.as_slice())?;

    Ok(state)
}

fn setup_scene(ctx: &WebGlRenderingContext, viewport: gl::Viewport) -> Result<(gl::Program, gl::GlState), String> {
    let program = display_program(&ctx)?;

    let vertices: [f32; 8] = [
        -0.9, -0.9,
        0.9, -0.9,
        0.9, 0.9,
        -0.9, 0.9
    ];
    let uvs: [f32; 8] = [
        0.0, 0.0,
        1.0, 0.0,
        1.0, 1.0,
        0.0, 1.0
    ];
    let indices: [u16; 6] = [0, 1, 2, 2, 3, 0];

    let state = setup_state(&ctx, viewport, vertices, uvs, indices)?;

    Ok((program, state))
}

fn animation_step(ctx: &WebGlRenderingContext, program: &gl::Program, state: &gl::GlState, time: u32) -> Result<(), String> {
    let uniforms: HashMap<_, _> = vec![
        ("tex", gl::UniformData::Texture("tex")),
        ("time", gl::UniformData::Scalar(time as f32)),
    ].into_iter().collect();

    state.run_mut(ctx, &program, &uniforms, "buf")?;

    let uniforms2: HashMap<_, _> = vec![
        ("tex", gl::UniformData::Texture("buf")),
        ("time", gl::UniformData::Scalar(time as f32)),
    ].into_iter().collect();

    state.run(ctx, &program, &uniforms2)?;

    Ok(())
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn prepare_canvas() -> Result<(WebGlRenderingContext, gl::Viewport), JsValue> {
    let document = window().document().ok_or("Expected document")?;
    let canvas = document.get_element_by_id("canvas").ok_or("Missig 'canvas' element")?;
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>()?;

    let context = canvas
        .get_context("webgl")?
        .ok_or("WebGl 1.0 not supported")?
        .dyn_into::<WebGlRenderingContext>()?;

    context.clear_color(0.0, 0.0, 0.0, 1.0);
    context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    Ok((context, gl::Viewport { w: canvas.width(), h: canvas.height() }))
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let (context, viewport) = prepare_canvas()?;

    let (p, state) = setup_scene(&context, viewport)?;

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut time = 0;
    *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
        if time > 300 {
            let _ = f.borrow_mut().take();
            return;
        }
        // todo: get delta since last frame?
        match animation_step(&context, &p, &state, time) {
            Ok(_) => (),
            Err(message) => log!("Error running animation step: {}", message),
        }
        time += 1;
        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut()>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    Ok(())
}