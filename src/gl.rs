use web_sys::*;
use js_sys;
use std::collections::HashMap;

type Ctx = WebGlRenderingContext;

#[derive(Clone)]
pub enum AttributeType {
    Vector(u8),
}

pub enum UniformType {
    Sampler2D,
    Float,
    Vector2,
}

#[derive(Clone)]
pub struct AttributeDescription {
    pub name: &'static str,
    pub location: Option<i32>,
    pub t: AttributeType,
}

pub struct UniformDescription {
    pub name: &'static str,
    pub location: Option<WebGlUniformLocation>,
    pub t: UniformType,
}

pub struct Program {
    program: WebGlProgram,
    attributes: Vec<AttributeDescription>,
    uniforms: Vec<UniformDescription>,
}

impl Program {
    pub fn new(
        ctx: &Ctx,
        vertex: &str,
        fragment: &str,
        uniforms: Vec<UniformDescription>,
        attributes: Vec<AttributeDescription>) -> Result<Program, &'static str> {

        let vertex_id = Program::shader(ctx, Ctx::VERTEX_SHADER, vertex)?;
        let fragment_id = Program::shader(ctx, Ctx::FRAGMENT_SHADER, fragment)?;

        let program = ctx.create_program().ok_or("Failed to create program")?;
        ctx.attach_shader(&program, &vertex_id);
        ctx.attach_shader(&program, &fragment_id);
        ctx.link_program(&program);

        let attributes = attributes.into_iter().map(|a| {
            AttributeDescription { location: Some(ctx.get_attrib_location(&program, a.name)), .. a }
        }).collect();

        let uniforms = uniforms.into_iter().flat_map(|u| {
            ctx.get_uniform_location(&program, u.name).map(|u_loc| UniformDescription { location: Some(u_loc), .. u })
        }).collect();

        Ok(Program { program, attributes, uniforms })
    }

    fn shader(ctx: &Ctx, shader_type: u32, source: &str) -> Result<WebGlShader, &'static str> {
        let shader = ctx
            .create_shader(shader_type).ok_or("Failed to create shader")?;
        ctx.shader_source(&shader, source);
        ctx.compile_shader(&shader);

        if ctx
            .get_shader_parameter(&shader, Ctx::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err("Failed to compile shader")
        }
    }
}

pub enum UniformData {
    Scalar(f32),
    Vector2([f32; 2]),
    Texture(&'static str)
}

pub struct GlState {
    textures: HashMap<&'static str, WebGlTexture>,
    vertex_buffers: HashMap<&'static str, WebGlBuffer>,
    element_buffer: Option<WebGlBuffer>,
    element_buffer_size: usize,
}

impl GlState {
    pub fn new() -> GlState {
        GlState {
            textures: HashMap::new(),
            vertex_buffers: HashMap::new(),
            element_buffer: None,
            element_buffer_size: 0
        }
    }

    pub fn vertex_buffer(&mut self, ctx: &Ctx, name: &'static str, data: &[u8]) -> Option<&mut Self> {
        let buffer = ctx.create_buffer()?;
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, Some(&buffer));
        ctx.buffer_data_with_u8_array(Ctx::ARRAY_BUFFER, data, Ctx::STATIC_DRAW);

        self.vertex_buffers.insert(name, buffer);
        Some(self)
    }

    pub fn vertex_buffer_v(&mut self, ctx: &Ctx, name: &'static str, data: &[f32]) -> Option<&mut Self> {
        let buffer = ctx.create_buffer()?;
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, Some(&buffer));

        unsafe {
            let vert_array = js_sys::Float32Array::view(&data);

            ctx.buffer_data_with_array_buffer_view(
                WebGlRenderingContext::ARRAY_BUFFER,
                &vert_array,
                WebGlRenderingContext::STATIC_DRAW,
            );
        }

        self.vertex_buffers.insert(name, buffer);
        Some(self)
    }

    pub fn element_buffer(&mut self, ctx: &Ctx, data: &[u8]) -> Option<&mut Self> {
        let buffer = ctx.create_buffer()?;
        ctx.bind_buffer(Ctx::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        ctx.buffer_data_with_u8_array(Ctx::ELEMENT_ARRAY_BUFFER, data, Ctx::STATIC_DRAW);

        self.element_buffer = Some(buffer);
        self.element_buffer_size = data.len();
        Some(self)
    }

    // pub fn element_buffer_v(&mut self, ctx: &Ctx, name: &'static str, data: &[u8]) -> Option<&mut Self> {
    //     let buffer = ctx.create_buffer()?;
    //     ctx.bind_buffer(Ctx::ELEMENT_ARRAY_BUFFER, Some(&buffer));

    //     unsafe {
    //         let vert_array = js_sys::Uint8Array::view(&data);

    //         ctx.buffer_data_with_array_buffer_view(
    //             WebGlRenderingContext::ELEMENT_ARRAY_BUFFER,
    //             &vert_array,
    //             WebGlRenderingContext::STATIC_DRAW,
    //         );
    //     }

    //     self.vertex_buffers.insert(name, buffer);
    //     self.element_buffer_size = data.len();
    //     Some(self)
    // }


    pub fn texture(&mut self, ctx: &Ctx, name: &'static str, data: &[u8], w: u32, h: u32) -> Option<&mut Self> {
        let tex = ctx.create_texture()?;
        ctx.bind_texture(Ctx::TEXTURE_2D, Some(&tex));
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Ctx::TEXTURE_2D,
            0,
            Ctx::RGBA as i32,
            w as i32,
            h as i32,
            0,
            Ctx::RGBA,
            Ctx::FLOAT,
            Some(data)
        ).ok()?;

        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MIN_FILTER, Ctx::NEAREST as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MAG_FILTER, Ctx::NEAREST as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_T, Ctx::CLAMP_TO_EDGE as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_S, Ctx::CLAMP_TO_EDGE as i32);

        self.textures.insert(name, tex);
        Some(self)
    }

    pub fn run(&self, ctx: &Ctx, program: &Program, uni_values: HashMap<&'static str, UniformData>) -> Option<&Self> {
        ctx.use_program(Some(&program.program));

        self.setup_attributes(ctx, &program.attributes)?;
        self.setup_uniforms(ctx, &program.uniforms, uni_values)?;

        ctx.draw_arrays(Ctx::TRIANGLES, 0, 3 as i32);
        // ctx.draw_elements_with_i32(
        //     Ctx::TRIANGLES,
        //     self.element_buffer_size as i32,
        //     Ctx::UNSIGNED_SHORT,
        //     0);

        Some(self)
    }

    fn setup_attributes(&self, ctx: &Ctx, attributes: &Vec<AttributeDescription>) -> Option<&Self> {
        let mut vert_array_idx = 0;
        for att in attributes {
            let idx = att.location? as u32;

            let buffer = self.vertex_buffers.get(att.name)?;
            ctx.bind_buffer(Ctx::ARRAY_BUFFER, Some(&buffer));

            let size = match att.t {
                AttributeType::Vector(size) => Some(size)
                // _ => None
            }? as i32;

            ctx.enable_vertex_attrib_array(vert_array_idx);
            ctx.vertex_attrib_pointer_with_i32(idx, size, Ctx::FLOAT, false, 0, 0);

            vert_array_idx += 1;
        }

        Some(self)
    }

    fn setup_uniforms(&self, ctx: &Ctx, uniforms: &Vec<UniformDescription>, uniform_values: HashMap<&'static str, UniformData>) -> Option<&Self> {
        let mut tex_inc = 0;

        for uni in uniforms {
            let loc = uni.location.clone()?;
            match uni.t {
                // todo: supply scalar
                UniformType::Float =>
                    match uniform_values.get(uni.name)? {
                        UniformData::Scalar(v) => ctx.uniform1f(Some(&loc), v.clone()),
                        _ => ()
                    },
                // todo: supply vector
                UniformType::Vector2 =>
                    match uniform_values.get(uni.name)? {
                        UniformData::Vector2(v) => ctx.uniform2fv_with_f32_array(Some(&loc), v),
                        _ => ()
                    },
                UniformType::Sampler2D => {
                    ctx.active_texture(Ctx::TEXTURE0 + tex_inc);
                    let tex = self.textures.get(uni.name)?;
                    ctx.bind_texture(Ctx::TEXTURE_2D, Some(&tex));
                    ctx.uniform1i(Some(&loc), tex_inc as i32);
                    tex_inc += 1;
                }
            }
        }
        Some(self)
    }
}