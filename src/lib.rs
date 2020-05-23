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
    // let indices_u8: [u8; 6] = [0, 1, 2, 2, 3, 0];

    let vb: Vec<u8> = vertices.into_iter().flat_map(|v| v.to_ne_bytes().to_vec()).collect();
    let eb: Vec<u8> = indices.into_iter().flat_map(|e| e.to_ne_bytes().to_vec()).collect();


    let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];

    state
        // .vertex_buffer(ctx, "position", vb.as_slice()).ok_or("Failed to create vertex buffer")?;
        .vertex_buffer_v(ctx, "position", &vertices).ok_or("Failed to create vertex buffer")?;
        // .element_buffer(ctx, eb.as_slice()).ok_or("Failed to create element buffer")?;


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

    log!("We tried to draw");

    Ok(())

    // let vert_shader = compile_shader(
    //     &context,
    //     WebGlRenderingContext::VERTEX_SHADER,
    //     r#"
    //     attribute vec4 position;
    //     void main() {
    //         gl_Position = position;
    //     }
    // "#,
    // )?;
    // let frag_shader = compile_shader(
    //     &context,
    //     WebGlRenderingContext::FRAGMENT_SHADER,
    //     r#"
    //     void main() {
    //         gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
    //     }
    // "#,
    // )?;
    // let program = link_program(&context, &vert_shader, &frag_shader)?;
    // context.use_program(Some(&program));

    // let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.7, 0.0];

    // let buffer = context.create_buffer().ok_or("failed to create buffer")?;
    // context.bind_buffer(WebGlRenderingContext::ARRAY_BUFFER, Some(&buffer));

    // // Note that `Float32Array::view` is somewhat dangerous (hence the
    // // `unsafe`!). This is creating a raw view into our module's
    // // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
    // // (aka do a memory allocation in Rust) it'll cause the buffer to change,
    // // causing the `Float32Array` to be invalid.
    // //
    // // As a result, after `Float32Array::view` we have to be very careful not to
    // // do any memory allocations before it's dropped.
    // unsafe {
    //     let vert_array = js_sys::Float32Array::view(&vertices);

    //     context.buffer_data_with_array_buffer_view(
    //         WebGlRenderingContext::ARRAY_BUFFER,
    //         &vert_array,
    //         WebGlRenderingContext::STATIC_DRAW,
    //     );
    // }

    // context.vertex_attrib_pointer_with_i32(0, 3, WebGlRenderingContext::FLOAT, false, 0, 0);
    // context.enable_vertex_attrib_array(0);

    // context.clear_color(0.0, 0.0, 0.0, 1.0);
    // context.clear(WebGlRenderingContext::COLOR_BUFFER_BIT);

    // context.draw_arrays(
    //     WebGlRenderingContext::TRIANGLES,
    //     0,
    //     (vertices.len() / 3) as i32,
    // );
    // Ok(())
}

// pub fn compile_shader(
//     context: &WebGlRenderingContext,
//     shader_type: u32,
//     source: &str,
// ) -> Result<WebGlShader, String> {
//     let shader = context
//         .create_shader(shader_type)
//         .ok_or_else(|| String::from("Unable to create shader object"))?;
//     context.shader_source(&shader, source);
//     context.compile_shader(&shader);

//     if context
//         .get_shader_parameter(&shader, WebGlRenderingContext::COMPILE_STATUS)
//         .as_bool()
//         .unwrap_or(false)
//     {
//         Ok(shader)
//     } else {
//         Err(context
//             .get_shader_info_log(&shader)
//             .unwrap_or_else(|| String::from("Unknown error creating shader")))
//     }
// }

// pub fn link_program(
//     context: &WebGlRenderingContext,
//     vert_shader: &WebGlShader,
//     frag_shader: &WebGlShader,
// ) -> Result<WebGlProgram, String> {
//     let program = context
//         .create_program()
//         .ok_or_else(|| String::from("Unable to create shader object"))?;

//     context.attach_shader(&program, vert_shader);
//     context.attach_shader(&program, frag_shader);
//     context.link_program(&program);

//     if context
//         .get_program_parameter(&program, WebGlRenderingContext::LINK_STATUS)
//         .as_bool()
//         .unwrap_or(false)
//     {
//         Ok(program)
//     } else {
//         Err(context
//             .get_program_info_log(&program)
//             .unwrap_or_else(|| String::from("Unknown error creating program object")))
//     }
// }
