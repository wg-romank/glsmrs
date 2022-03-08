use mesh::Mesh;
use web_sys::*;
use std::{collections::HashMap, rc::Rc};

pub mod util;
pub mod texture;
pub mod mesh;

use crate::texture::*;

type Ctx = WebGlRenderingContext;

pub enum AttributeType {
    Scalar, Vector2, Vector3, Vector4
}

impl AttributeType {
    fn num_components(&self) -> i32 {
        match *self {
            AttributeType::Scalar => 1,
            AttributeType::Vector2 => 2,
            AttributeType::Vector3 => 3,
            AttributeType::Vector4 => 4,
        }
    }
}

pub enum UniformType {
    Sampler2D,
    Float,
    Vector2,
    Vector4,
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
    ctx: Ctx,
    program: WebGlProgram,
    uniforms: Vec<UniformDescription>,
}

impl Program {
    pub fn new(
        ctx: &Ctx,
        vertex: &str,
        fragment: &str,
        unis: Vec<UniformDescription>,
    ) -> Result<Program, String> {
        Program::new_with_mode(ctx, vertex, fragment, unis)
    }

    pub fn new_with_mode(
        ctx: &Ctx,
        vertex: &str,
        fragment: &str,
        unis: Vec<UniformDescription>,
    ) -> Result<Program, String> {

        let vertex_id = Program::shader(ctx, Ctx::VERTEX_SHADER, vertex)?;
        let fragment_id = Program::shader(ctx, Ctx::FRAGMENT_SHADER, fragment)?;

        let program = ctx.create_program().ok_or("Failed to create program")?;
        ctx.attach_shader(&program, &vertex_id);
        ctx.attach_shader(&program, &fragment_id);
        ctx.link_program(&program);

        // todo: unbound uniforms
        let uniforms = unis.into_iter().map(|u| {
            match ctx.get_uniform_location(&program, u.name) {
                Some(u_loc) => Ok(UniformDescription { location: Some(u_loc), .. u }),
                None => Err(format!("Failed to locate uniform {}", u.name))
            }
        }).collect::<Result<_, _>>()?;

        Ok(Program { ctx: ctx.clone(), program, uniforms })
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

impl Drop for Program {
    fn drop(&mut self) {
        self.ctx.delete_program(Some(&self.program));
    }
}

pub enum UniformData {
    Scalar(f32),
    Vector2([f32; 2]),
    Vector4([f32; 4]),
    Texture(Rc<UploadedTexture>),
}

pub struct GlState {
    ctx: Rc<Ctx>,
    viewport: Viewport,
    objects: Vec<Mesh>,
}

impl GlState {
    pub fn new(ctx: &Rc<Ctx>, viewport: Viewport) -> GlState {
        GlState {
            ctx: ctx.clone(),
            viewport,
            objects: vec![],
        }
    }

    pub fn add_mesh(mut self, obj: Mesh) -> GlState {
        self.objects.push(obj);
        self
    }

    pub fn run(&self, program: &Program, uni_values: &HashMap<&'static str, UniformData>) -> Result<&Self, String> {
        for obj in self.objects.iter() {
            self.ctx.use_program(Some(&program.program));
            self.setup_uniforms(&program.uniforms, uni_values)?;
            obj.draw(program)?;
        }

        Ok(self)
    }

    pub fn run_mut<'a>(
        &self, program: &Program,
        uni_values: &HashMap<&'static str, UniformData>,
        output: &mut Framebuffer,
    ) -> Result<&Self, String> {
        output.bind(&self.ctx);

        self.run(&program, &uni_values)?;

        self.ctx.bind_framebuffer(Ctx::FRAMEBUFFER, None);
        self.ctx.viewport(0, 0, self.viewport.w as i32, self.viewport.h as i32);

        Ok(self)
    }

    // todo: keep index to only update when new textures are coming?
    fn setup_uniforms(&self, uniforms: &Vec<UniformDescription>, uniform_values: &HashMap<&'static str, UniformData>) -> Result<&Self, String> {
        let mut tex_inc = 0;

        for uni in uniforms {
            let loc = uni.location.clone().ok_or(format!("Location for uniform {} is not set", uni.name))?;
            match uni.t {
                UniformType::Float =>
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for scalar uniform {}", uni.name))? {
                        UniformData::Scalar(v) => self.ctx.uniform1f(Some(&loc), v.clone()),
                        _ => ()
                    },
                UniformType::Vector2 =>
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for vector uniform {}", uni.name))? {
                        UniformData::Vector2(v) => self.ctx.uniform2fv_with_f32_array(Some(&loc), v),
                        _ => ()
                    },
                UniformType::Vector4 =>
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for vector uniform {}", uni.name))? {
                        UniformData::Vector4(v) => self.ctx.uniform4fv_with_f32_array(Some(&loc), v),
                        _ => ()
                    },
                UniformType::Sampler2D => {
                    match uniform_values.get(uni.name).ok_or(format!("Missing value for texture uniform {}", uni.name))? {
                        UniformData::Texture(tex) => {
                            self.ctx.active_texture(Ctx::TEXTURE0 + tex_inc);
                            tex.bind(&self.ctx);

                            // todo: double check on safely disposing uniforms data
                            self.ctx.uniform1i(Some(&loc), tex_inc as i32);
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
