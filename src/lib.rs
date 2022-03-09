use mesh::Mesh;
use std::{collections::HashMap, rc::Rc, ops::Deref};
use web_sys::*;

pub mod mesh;
pub mod texture;
pub mod util;

use crate::texture::*;

#[derive(Clone)]
pub struct Ctx(Rc<WebGlRenderingContext>);

impl Ctx {
    pub fn new(ctx: WebGlRenderingContext) -> Self {
        Self(Rc::new(ctx))
    }
}

impl Deref for Ctx {
    type Target = WebGlRenderingContext;

    fn deref(&self) -> &WebGlRenderingContext {
        &self.0
    }
}

pub type GL = WebGlRenderingContext;

pub enum AttributeType {
    Scalar,
    Vector2,
    Vector3,
    Vector4,
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

pub struct Program {
    ctx: Ctx,
    program: WebGlProgram,
}

impl Program {
    pub fn new(ctx: &Ctx, vertex: &str, fragment: &str) -> Result<Program, String> {
        Program::new_with_mode(ctx, vertex, fragment)
    }

    pub fn new_with_mode(ctx: &Ctx, vertex: &str, fragment: &str) -> Result<Program, String> {
        let vertex_id = Program::shader(ctx, GL::VERTEX_SHADER, vertex)?;
        let fragment_id = Program::shader(ctx, GL::FRAGMENT_SHADER, fragment)?;

        let program = ctx.create_program().ok_or("Failed to create program")?;
        ctx.attach_shader(&program, &vertex_id);
        ctx.attach_shader(&program, &fragment_id);
        ctx.link_program(&program);

        Ok(Program {
            ctx: ctx.clone(),
            program,
        })
    }

    fn shader(ctx: &Ctx, shader_type: u32, source: &str) -> Result<WebGlShader, String> {
        let shader = ctx
            .create_shader(shader_type)
            .ok_or(format!("Failed to create shader {}", shader_type))?;
        ctx.shader_source(&shader, source);
        ctx.compile_shader(&shader);

        if ctx
            .get_shader_parameter(&shader, GL::COMPILE_STATUS)
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
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Texture(Rc<UploadedTexture>),
}

pub enum RenderTarget<'a, C, D> {
    Framebuffer(&'a mut Framebuffer<C, D>),
    Display(Viewport),
}

pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self {}
    }

    pub fn shade<'a, C, D>(
        &self,
        ctx: &Ctx,
        program: &Program,
        uni_values: &HashMap<&'static str, UniformData>,
        objects: Vec<&Mesh>,
        output: RenderTarget<'a, C, D>,
    ) -> Result<&Self, String> {
        match output {
            RenderTarget::Framebuffer(fb) => fb.bind(),
            RenderTarget::Display(vp) => {
                ctx.bind_framebuffer(GL::FRAMEBUFFER, None);
                vp.set(&ctx);
            }
        }

        ctx.use_program(Some(&program.program));
        self.set_uniforms(ctx, &program, uni_values)?;

        for obj in objects.iter() {
            obj.draw(program)?;
        }

        Ok(self)
    }

    fn set_uniforms(
        &self,
        ctx: &Ctx,
        program: &Program,
        uniform_values: &HashMap<&'static str, UniformData>,
    ) -> Result<&Self, String> {
        let mut tex_inc = 0;
        for (&name, uni_val) in uniform_values.iter() {
            if let Some(loc) = ctx.get_uniform_location(&program.program, name) {
                match uni_val {
                    UniformData::Scalar(v) => ctx.uniform1f(Some(&loc), v.clone()),
                    UniformData::Vector2(v) => ctx.uniform2fv_with_f32_array(Some(&loc), v),
                    UniformData::Vector3(v) => ctx.uniform3fv_with_f32_array(Some(&loc), v),
                    UniformData::Vector4(v) => ctx.uniform4fv_with_f32_array(Some(&loc), v),
                    UniformData::Texture(tex) => {
                        ctx.active_texture(GL::TEXTURE0 + tex_inc);
                        tex.bind();

                        // todo: double check on safely disposing uniforms data
                        ctx.uniform1i(Some(&loc), tex_inc as i32);
                        tex_inc += 1;
                    }
                }
            }
        }
        Ok(self)
    }
}
