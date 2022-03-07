use web_sys::{WebGlTexture, WebGlFramebuffer};

use crate::Ctx;

#[derive(Clone)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Self {
        Self { x: 0, y: 0, w: width as i32, h: height as i32 }
    }
}

#[derive(Clone)]
pub struct UploadedTexture {
    handle: WebGlTexture,
}

impl UploadedTexture {
    pub fn bind(&self, ctx: &Ctx) {
        ctx.bind_texture(Ctx::TEXTURE_2D, Some(&self.handle));
    }
}

// todo:
// impl Drop for Rc<UploadedTexture> {
//     fn drop(&mut self) {
//         self.ctx.delete_texture(Some(&self.handle));
//     }
// }

pub struct TextureSpec {
    pub color_format: u32,
    pub dimensions: [u32; 2],
    pub interpolation_min: u32,
    pub interpolation_mag: u32,
    pub wrap_t: u32,
    pub wrap_s: u32,
}

impl TextureSpec {
    pub fn upload(&self, ctx: &Ctx, data: Option<&[u8]>) -> Result<UploadedTexture, String> {
        let handle = ctx.create_texture().ok_or(format!("Failed to create texture"))?;
        ctx.bind_texture(Ctx::TEXTURE_2D, Some(&handle));
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            Ctx::TEXTURE_2D,
            0,
            self.color_format as i32,
            self.dimensions[0] as i32,
            self.dimensions[1] as i32,
            0,
            self.color_format,
            Ctx::UNSIGNED_BYTE,
            data,
        ).map_err(|e| format!("Failed to send image data {:?}", e))?;

        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MIN_FILTER, self.interpolation_min as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_MAG_FILTER, self.interpolation_mag as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_T, self.wrap_t as i32);
        ctx.tex_parameteri(Ctx::TEXTURE_2D, Ctx::TEXTURE_WRAP_S, self.wrap_s as i32);

        Ok(
            UploadedTexture {
                handle
            }
        )
    }
}

pub struct Framebuffer {
    viewport: Viewport,
    handle: WebGlFramebuffer,
    pub color_slot: Option<UploadedTexture>,
    pub depth_slot: Option<UploadedTexture>,
}

impl Framebuffer {
    pub fn new(ctx: &Ctx, viewport: Viewport) -> Result<Self, String> {
        let handle = ctx.create_framebuffer().ok_or(format!("Failed to create frame buffer"))?;

        Ok(
            Self {
                viewport,
                handle,
                color_slot: None,
                depth_slot: None,
            }
        )
    }

    pub fn with_color_slot(mut self, ctx: &Ctx, tex: UploadedTexture) -> Self {
        self.bind(ctx);
        ctx.framebuffer_texture_2d(Ctx::FRAMEBUFFER, Ctx::COLOR_ATTACHMENT0, Ctx::TEXTURE_2D, Some(&tex.handle), 0);
        self.color_slot = Some(tex);

        self
    }

    pub fn bind(&self, ctx: &Ctx) {
        ctx.bind_framebuffer(Ctx::FRAMEBUFFER, Some(&self.handle));
        ctx.viewport(self.viewport.x, self.viewport.y, self.viewport.w, self.viewport.h);
    }
}

// todo:
// impl Drop for Rc<Framebuffer> {
//     fn drop(&mut self) {
//         self.ctx.delete_framebuffer(Some(&self.handle));
//     }
// }