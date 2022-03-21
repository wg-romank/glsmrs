use mesh::Mesh;
use util::get_ctx;
use std::{collections::HashMap, rc::Rc, ops::Deref};
use web_sys::*;

pub mod mesh;
pub mod texture;
pub mod util;
pub mod attributes;

use crate::texture::*;

#[derive(Clone)]
pub struct Ctx(Rc<WebGlRenderingContext>);

impl Ctx {
    pub fn from(canvas_name: &str) -> Result<Self, String> {
        let ctx = get_ctx(canvas_name, "webgl").map_err(|e| format!("{:?}", e))?;
        Self::new(ctx)
    }
    pub fn new(ctx: WebGlRenderingContext) -> Result<Self, String> {
        ctx.get_extension("WEBGL_depth_texture").map_err(|e| format!("no depth textures available {:?}", e))?;
        ctx.get_extension("OES_texture_float").map_err(|e| format!("no float textures available {:?}", e))?;
        ctx.enable(GL::DEPTH_TEST);
        ctx.enable(GL::CULL_FACE);

        Ok(Self(Rc::new(ctx)))
    }
}

impl Deref for Ctx {
    type Target = WebGlRenderingContext;

    fn deref(&self) -> &WebGlRenderingContext {
        &self.0
    }
}

pub type GL = WebGlRenderingContext;

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
            Err(format!("Failed to compile shader {:?}", ctx.get_shader_info_log(&shader)))
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        self.ctx.delete_program(Some(&self.program));
    }
}

pub enum UniformData<'a> {
    Scalar(f32),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    Matrix4([f32; 16]),
    Texture(&'a mut UploadedTexture),
}

pub struct Pipeline {
    ctx: Ctx,
    clear_color: Option<[f32; 4]>,
    clear_depth: Option<f32>,
    clear_stencil: Option<i32>,
}

impl Pipeline {
    pub fn new(ctx: &Ctx) -> Self {
        let s = Self {
            ctx: ctx.clone(),
            clear_color: Some([0., 0., 0., 1.]),
            clear_depth: Some(1.),
            clear_stencil: Some(0),
        };

        if let Some(col) = s.clear_color {
            ctx.clear_color(col[0], col[1], col[2], col[3]);
        }
        if let Some(d) = s.clear_depth {
            ctx.clear_depth(d);
        }
        if let Some(s) = s.clear_stencil {
            ctx.clear_stencil(s);
        };

        s
    }

    pub fn shade<'a, T: Framebuffer>(
        &mut self,
        program: &Program,
        uni_values: HashMap<&'static str, UniformData>,
        objects: Vec<&'a mut Mesh>,
        output: &'a mut T,
    ) -> Result<&Self, String> {
        output.bind();

        if self.clear_color.is_some() {
            self.ctx.clear(GL::COLOR_BUFFER_BIT);
        }
        if self.clear_depth.is_some() {
            self.ctx.clear(GL::DEPTH_BUFFER_BIT);
        }
        if self.clear_stencil.is_some() {
            self.ctx.clear(GL::STENCIL_BUFFER_BIT);
        }

        self.ctx.use_program(Some(&program.program));
        self.set_uniforms(&program, uni_values)?;

        for obj in objects.into_iter() {
            obj.draw(program)?;
        }

        Ok(self)
    }

    fn set_uniforms(
        &self,
        program: &Program,
        uniform_values: HashMap<&'static str, UniformData>,
    ) -> Result<&Self, String> {
        let mut tex_inc = 0;
        for (name, uni_val) in uniform_values.into_iter() {
            if let Some(loc) = self.ctx.get_uniform_location(&program.program, name) {
                match uni_val {
                    UniformData::Scalar(v) => self.ctx.uniform1f(Some(&loc), v.clone()),
                    UniformData::Vector2(v) => self.ctx.uniform2fv_with_f32_array(Some(&loc), &v),
                    UniformData::Vector3(v) => self.ctx.uniform3fv_with_f32_array(Some(&loc), &v),
                    UniformData::Vector4(v) => self.ctx.uniform4fv_with_f32_array(Some(&loc), &v),
                    UniformData::Matrix4(m) => self.ctx.uniform_matrix4fv_with_f32_array(Some(&loc), false, &m),
                    UniformData::Texture(tex) => {
                        self.ctx.active_texture(GL::TEXTURE0 + tex_inc);
                        tex.bind();

                        // todo: double check on safely disposing uniforms data
                        self.ctx.uniform1i(Some(&loc), tex_inc as i32);
                        tex_inc += 1;
                    }
                }
            }
        }
        Ok(self)
    }
}
