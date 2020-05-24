use web_sys::*;
use std::collections::HashMap;

type Ctx = WebGlRenderingContext;

#[derive(Clone)]
pub enum AttributeType {
    Vector2, Vector3, Vector4
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

impl AttributeDescription {
    pub fn new(name: &'static str, t: AttributeType) -> AttributeDescription {
        AttributeDescription { name, location: None, t }
    }
}

pub struct UniformDescription {
    pub name: &'static str,
    pub location: Option<WebGlUniformLocation>,
    pub t: UniformType,
}

impl UniformDescription {
    pub fn new (name: &'static str, t: UniformType) -> UniformDescription {
        UniformDescription { name, location: None, t }
    }
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
        unis: Vec<UniformDescription>,
        attrs: Vec<AttributeDescription>) -> Result<Program, String> {

        let vertex_id = Program::shader(ctx, Ctx::VERTEX_SHADER, vertex)?;
        let fragment_id = Program::shader(ctx, Ctx::FRAGMENT_SHADER, fragment)?;

        let program = ctx.create_program().ok_or("Failed to create program")?;
        ctx.attach_shader(&program, &vertex_id);
        ctx.attach_shader(&program, &fragment_id);
        ctx.link_program(&program);

        let (attributes_r, errors): (Vec<_>, Vec<_>) = attrs.into_iter().map(|a| {
            let loc = ctx.get_attrib_location(&program, a.name);
            if loc >= 0 {
                Ok(AttributeDescription { location: Some(loc), .. a })
            } else {
                Err(format!("Failed to locate attrib {}", a.name))
            }
        }).partition(Result::is_ok);

        if !errors.is_empty() {
            let msgs: Vec<String> = errors.into_iter().flat_map(|e| e.err()).collect();

            Err(msgs.join("-"))
        } else {
            let attributes = attributes_r.into_iter().flat_map(|a| a).collect();

            // todo: similar checks for uniforms
            let uniforms: Vec<UniformDescription> = unis.into_iter().flat_map(|u| {
                ctx.get_uniform_location(&program, u.name).map(|u_loc| UniformDescription { location: Some(u_loc), .. u })
            }).collect();

            Ok(Program { program, attributes, uniforms })
        }
    }

    fn shader(ctx: &Ctx, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let shader = ctx
            .create_shader(shader_type).ok_or(format!("Failed to create shader {}", shader_type))?;
        ctx.shader_source(&shader, source);
        ctx.compile_shader(&shader);

        if ctx
            .get_shader_parameter(&shader, Ctx::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(format!("Failed to compile shader {}", shader_type))
        }
    }
}

pub struct Viewport {
    pub w: u32,
    pub h: u32
}

pub struct TextureSpec {
    viewport: Viewport,
    handle: WebGlTexture
}

pub enum UniformData {
    Scalar(f32),
    Vector2([f32; 2]),
    Texture(&'static str)
}

pub struct GlState {
    viewport: Viewport,
    textures: HashMap<&'static str, TextureSpec>,
    vertex_buffers: HashMap<&'static str, WebGlBuffer>,
    element_buffer: Option<WebGlBuffer>,
    element_buffer_size: usize,
}

impl GlState {
    pub fn new(viewport: Viewport) -> GlState {
        GlState {
            viewport,
            textures: HashMap::new(),
            vertex_buffers: HashMap::new(),
            element_buffer: None,
            element_buffer_size: 0
        }
    }

    pub fn vertex_buffer(&mut self, ctx: &Ctx, name: &'static str, data: &[u8]) -> Result<&mut Self, String> {
        let buffer = ctx.create_buffer().ok_or(format!("Failed to create buffer for {}", name))?;
        ctx.bind_buffer(Ctx::ARRAY_BUFFER, Some(&buffer));
        ctx.buffer_data_with_u8_array(Ctx::ARRAY_BUFFER, data, Ctx::STATIC_DRAW);

        self.vertex_buffers.insert(name, buffer);
        Ok(self)
    }

    // todo: should be put to run & draw?
    pub fn element_buffer(&mut self, ctx: &Ctx, data: &[u8]) -> Result<&mut Self, String> {
        let buffer = ctx.create_buffer().ok_or("Failed to create element buffer")?;
        ctx.bind_buffer(Ctx::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        ctx.buffer_data_with_u8_array(Ctx::ELEMENT_ARRAY_BUFFER, data, Ctx::STATIC_DRAW);

        self.element_buffer = Some(buffer);
        self.element_buffer_size = data.len() / 2; // assuming UNSIGNED_SHORTS
        Ok(self)
    }

    pub fn texture(&mut self, ctx: &Ctx, name: &'static str, data: Option<&[u8]>, w: u32, h: u32) -> Result<&mut Self, String> {
        let tex = ctx.create_texture().ok_or(format!("Failed to create texture for {}", name))?;
        ctx.bind_texture(Ctx::TEXTURE_2D, Some(&tex));
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Ctx::TEXTURE_2D,
            0,
            Ctx::RGBA as i32,
            w as i32,
            h as i32,
            0,
            Ctx::RGBA,
            Ctx::UNSIGNED_BYTE,
            data
        ).map_err(|e| format!("Failed to send image data for {} {:?}", name, e))?;

        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MIN_FILTER, Ctx::NEAREST as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MAG_FILTER, Ctx::NEAREST as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_T, Ctx::CLAMP_TO_EDGE as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_S, Ctx::CLAMP_TO_EDGE as i32);

        self.textures.insert(name, TextureSpec {
            viewport: Viewport { w, h },
            handle: tex
        });
        Ok(self)
    }

    pub fn run(&self, ctx: &Ctx, program: &Program, uni_values: &HashMap<&'static str, UniformData>) -> Result<&Self, String> {
        if self.element_buffer.is_none() {
            Err("Element buffer is not set")?
        }

        ctx.use_program(Some(&program.program));

        self.setup_attributes(ctx, &program.attributes)?;
        self.setup_uniforms(ctx, &program.uniforms, uni_values)?;

        ctx.draw_elements_with_i32(
            Ctx::TRIANGLES,
            self.element_buffer_size as i32,
            Ctx::UNSIGNED_SHORT,
            0);

        Ok(self)
    }

    pub fn run_mut(&mut self, ctx: &Ctx, program: &Program, uni_values: &HashMap<&'static str, UniformData>,
                   name: &'static str) -> Result<&mut Self, String> {
        let tex = self.textures.get(name).ok_or(format!("Can't render to {} no such texture", name))?;

        let fb = ctx.create_framebuffer().ok_or(format!("Failed to create frame buffer for {}", name))?;
        ctx.bind_framebuffer(Ctx::FRAMEBUFFER, Some(&fb));
        ctx.framebuffer_texture_2d(Ctx::FRAMEBUFFER, Ctx::COLOR_ATTACHMENT0, Ctx::TEXTURE_2D, Some(&tex.handle), 0);
        ctx.viewport(0, 0, tex.viewport.w as i32, tex.viewport.h as i32);

        self.run(&ctx, &program, &uni_values)?;

        ctx.bind_framebuffer(Ctx::FRAMEBUFFER, None);
        ctx.viewport(0, 0, self.viewport.w as i32, self.viewport.h as i32);

        Ok(self)
    }

    fn setup_attributes(&self, ctx: &Ctx, attributes: &Vec<AttributeDescription>) -> Result<&Self, String> {
        let mut vert_array_idx = 0;
        for att in attributes {
            let idx = att.location.ok_or(format!("Location for attribute {} is not set", att.name))? as u32;

            let buffer = self.vertex_buffers.get(att.name).ok_or(format!("Vertex buffer for {} is not set", att.name))?;
            ctx.bind_buffer(Ctx::ARRAY_BUFFER, Some(&buffer));

            let size: i32 = match att.t {
                AttributeType::Vector2 => 2,
                AttributeType::Vector3 => 3,
                AttributeType::Vector4 => 4,
            } as i32;

            ctx.enable_vertex_attrib_array(vert_array_idx);
            ctx.vertex_attrib_pointer_with_i32(idx, size, Ctx::FLOAT, false, 0, 0);

            vert_array_idx += 1;
        }

        Ok(self)
    }

    // todo: keep index to only update when new textures are coming?
    fn setup_uniforms(&self, ctx: &Ctx, uniforms: &Vec<UniformDescription>, uniform_values: &HashMap<&'static str, UniformData>) -> Result<&Self, String> {
        let mut tex_inc = 0;

        for uni in uniforms {
            let loc = uni.location.clone().ok_or(format!("Location for uniform {} is not set", uni.name))?;
            match uni.t {
                UniformType::Float =>
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for scalar uniform {}", uni.name))? {
                        UniformData::Scalar(v) => ctx.uniform1f(Some(&loc), v.clone()),
                        _ => ()
                    },
                UniformType::Vector2 =>
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for vector uniform {}", uni.name))? {
                        UniformData::Vector2(v) => ctx.uniform2fv_with_f32_array(Some(&loc), v),
                        _ => ()
                    },
                UniformType::Sampler2D => {
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for texture uniform {}", uni.name))? {
                        UniformData::Texture(name) => {
                            ctx.active_texture(Ctx::TEXTURE0 + tex_inc);
                            let tex = self.textures.get(name).ok_or(format!("Missing texture {} references from uniform {}", name, uni.name))?;
                            ctx.bind_texture(Ctx::TEXTURE_2D, Some(&tex.handle));
                            ctx.uniform1i(Some(&loc), tex_inc as i32);
                            tex_inc += 1;
                        },
                        _ => ()
                    }
                }
            }
        }
        Ok(self)
    }
}