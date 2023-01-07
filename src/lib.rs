use mesh::Mesh;
use std::sync::Arc;
use std::ops::Deref;
use glow::Context;
use glow::HasContext;
use glow as GL;

pub mod mesh;
pub mod texture;
pub mod util;
pub mod attributes;

use crate::texture::*;

#[derive(Clone)]
pub struct Ctx(Arc<Context>);

impl Ctx {
    pub fn fromarc(glow: Arc<Context>) -> Self {
        Self(glow)
    }
    pub fn from(glow: Context) -> Self {
        // let ctx = get_ctx(canvas_name, "webgl").map_err(|e| format!("{:?}", e))?;
        Self(Arc::new(glow))
    }
    // pub fn new() -> Result<Self, String> {
    //     ctx.get_extension("WEBGL_depth_texture").map_err(|e| format!("no depth textures available {:?}", e))?;
    //     ctx.get_extension("OES_texture_float").map_err(|e| format!("no float textures available {:?}", e))?;
    //     ctx.enable(GL::DEPTH_TEST);
    //     ctx.enable(GL::CULL_FACE);
    //     let gl: Context = ctx.into();

    //     Ok(Self(Rc::new(gl)))
    // }
}

impl Deref for Ctx {
    type Target = Context;

    fn deref(&self) -> &Context {
        &self.0
    }
}

pub struct Program {
    ctx: Ctx,
    program: glow::Program,
}

impl Program {
    pub unsafe fn new(ctx: &Ctx, vertex: &str, fragment: &str) -> Result<Program, String> {
        let vertex_id = Program::shader(ctx, GL::VERTEX_SHADER, vertex)?;
        let fragment_id = Program::shader(ctx, GL::FRAGMENT_SHADER, fragment)?;

        let program = ctx.create_program()?;
        ctx.attach_shader(program, vertex_id);
        ctx.attach_shader(program, fragment_id);
        ctx.link_program(program);

        Ok(Program {
            ctx: ctx.clone(),
            program,
        })
    }

    unsafe fn shader(ctx: &Ctx, shader_type: u32, source: &str) -> Result<glow::Shader, String> {
        let shader = ctx
            .create_shader(shader_type)?;
        ctx.shader_source(shader, source);
        ctx.compile_shader(shader);

        if ctx.get_shader_compile_status(shader) {
            Ok(shader)
        } else {
            Err(format!("Failed to compile shader {:?}", ctx.get_shader_info_log(shader)))
        }
    }
}

impl Drop for Program {
    fn drop(&mut self) {
        unsafe {
            self.ctx.delete_program(self.program);
        }
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
    pub fn newe(ctx: &Ctx) -> Self {
        Self {
            ctx: ctx.clone(),
            clear_color: None,
            clear_depth: None,
            clear_stencil: None,
        }
    }
    pub unsafe fn new(ctx: &Ctx) -> Self {
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
            ctx.clear_depth_f32(d);
        }
        if let Some(s) = s.clear_stencil {
            ctx.clear_stencil(s);
        };

        s
    }

    pub unsafe fn shade<'a, T, U>(
        &mut self,
        program: &Program,
        uni_values: U,
        objects: Vec<&mut Mesh>,
        output: &'a mut T,
    ) -> Result<&Self, String> where
        T: Framebuffer,
        U: IntoIterator<Item = (&'a str, UniformData<'a>)>
    {
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

        self.ctx.use_program(Some(program.program));
        self.set_uniforms(program, uni_values)?;

        for obj in objects {
            obj.draw(program)?;
        }

        Ok(self)
    }

    unsafe fn set_uniforms<'a, U>(
        &self,
        program: &Program,
        uniform_values: U,
    ) -> Result<&Self, String> where
        U: IntoIterator<Item = (&'a str, UniformData<'a>)>
    {
        let mut tex_inc = 0;
        for (name, uni_val) in uniform_values {
            if let Some(loc) = self.ctx.get_uniform_location(program.program, name) {
                match uni_val {
                    UniformData::Scalar(v) => self.ctx.uniform_1_f32(Some(&loc), v),
                    UniformData::Vector2(v) => self.ctx.uniform_2_f32_slice(Some(&loc), &v),
                    UniformData::Vector3(v) => self.ctx.uniform_3_f32_slice(Some(&loc), &v),
                    UniformData::Vector4(v) => self.ctx.uniform_4_f32_slice(Some(&loc), &v),
                    UniformData::Matrix4(m) => self.ctx.uniform_matrix_4_f32_slice(Some(&loc), false, &m),
                    UniformData::Texture(tex) => {
                        self.ctx.active_texture(GL::TEXTURE0 + tex_inc);
                        tex.bind();

                        // todo: double check on safely disposing uniforms data
                        self.ctx.uniform_1_i32(Some(&loc), tex_inc as i32);
                        tex_inc += 1;
                    }
                }
            }
        }
        Ok(self)
    }
}
