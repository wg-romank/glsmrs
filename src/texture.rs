use std::ops::Deref;

use web_sys::{WebGlFramebuffer, WebGlTexture};

use crate::{GL, Ctx};

#[derive(Clone, Copy)]
pub struct Viewport {
    pub x: i32,
    pub y: i32,
    pub w: i32,
    pub h: i32,
}

impl Viewport {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            x: 0,
            y: 0,
            w: width as i32,
            h: height as i32,
        }
    }

    pub fn set(&self, ctx: &Ctx) {
        ctx.viewport(
            self.x,
            self.y,
            self.w,
            self.h,
        );
    }
}

#[derive(Clone, Copy)]
pub struct ColorFormat(pub u32);

impl Into<i32> for ColorFormat {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

impl Into<u32> for ColorFormat {
    fn into(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy)]
pub struct InterpolationMin(pub u32);

impl Into<i32> for InterpolationMin {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct InterpolationMag(pub u32);

impl Into<i32> for InterpolationMag {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct WrapT(pub u32);

impl Into<i32> for WrapT {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct WrapS(pub u32);


impl Into<i32> for WrapS {
    fn into(self) -> i32 {
        self.0 as i32
    }
}

pub struct TextureSpec {
    pub color_format: ColorFormat,
    pub dimensions: [u32; 2],
    pub interpolation_min: InterpolationMin,
    pub interpolation_mag: InterpolationMag,
    pub wrap_t: WrapT,
    pub wrap_s: WrapS,
}

impl TextureSpec {
    pub fn new(color_format: ColorFormat, dimensions: [u32; 2]) -> Self {
        Self {
            color_format,
            dimensions,
            interpolation_min: InterpolationMin(GL::LINEAR),
            interpolation_mag: InterpolationMag(GL::LINEAR),
            wrap_t: WrapT(GL::CLAMP_TO_EDGE),
            wrap_s: WrapS(GL::CLAMP_TO_EDGE),
        }
    }

    pub fn pixel(color_format: ColorFormat, dimensions: [u32; 2]) -> Self {
        Self {
            color_format,
            dimensions,
            interpolation_min: InterpolationMin(GL::NEAREST),
            interpolation_mag: InterpolationMag(GL::NEAREST),
            wrap_t: WrapT(GL::CLAMP_TO_EDGE),
            wrap_s: WrapS(GL::CLAMP_TO_EDGE),
        }
    }

    pub fn wrap_t(mut self, wrap: WrapT) -> Self {
        self.wrap_t = wrap;
        self
    }

    pub fn wrap_s(mut self, wrap: WrapS) -> Self {
        self.wrap_s = wrap;
        self
    }

    pub fn upload_rgba(&self, ctx: &Ctx, data: &[[f32; 4]]) -> Result<UploadedTexture, String> {
        let packed = data.iter().flat_map(|e| e.iter().flat_map(|ee| ee.to_be_bytes())).collect::<Vec<u8>>();
        self.upload(&ctx, Some(&packed))
    }

    pub fn upload(&self, ctx: &Ctx, data: Option<&[u8]>) -> Result<UploadedTexture, String> {
        let handle = ctx
            .create_texture()
            .ok_or(format!("Failed to create texture"))?;
        ctx.bind_texture(GL::TEXTURE_2D, Some(&handle));
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            GL::TEXTURE_2D,
            0,
            self.color_format.into(),
            self.dimensions[0] as i32,
            self.dimensions[1] as i32,
            0,
            self.color_format.into(),
            GL::UNSIGNED_BYTE,
            data,
        )
        .map_err(|e| format!("Failed to send image data {:?}", e))?;

        ctx.tex_parameteri(
            GL::TEXTURE_2D,
            GL::TEXTURE_MIN_FILTER,
            self.interpolation_min.into(),
        );
        ctx.tex_parameteri(
            GL::TEXTURE_2D,
            GL::TEXTURE_MAG_FILTER,
            self.interpolation_mag.into(),
        );
        ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, self.wrap_t.into());
        ctx.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, self.wrap_s.into());

        Ok(UploadedTexture {
            ctx: ctx.clone(),
            handle,
        })
    }
}

pub struct UploadedTexture {
    ctx: Ctx,
    handle: WebGlTexture,
}

impl UploadedTexture {
    pub fn bind(&self) {
        self.ctx.bind_texture(GL::TEXTURE_2D, Some(&self.handle));
    }
}

impl Drop for UploadedTexture {
    fn drop(&mut self) {
        self.ctx.delete_texture(Some(&self.handle));
    }
}


pub struct Framebuffer<C, D> {
    ctx: Ctx,
    viewport: Viewport,
    handle: WebGlFramebuffer,
    pub color_slot: C,
    pub depth_slot: D,
}

impl<C, D> Framebuffer<C, D> {
    pub fn dimensions(&self) -> [f32; 2] {
        [self.viewport.w as f32, self.viewport.h as f32]
    }

    pub fn bind(&self) {
        self.ctx.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.handle));
        self.viewport.set(&self.ctx);
    }
}

impl Framebuffer<(), ()> {
    pub fn new(ctx: &Ctx, viewport: Viewport) -> Result<Self, String> {
        let handle = ctx
            .create_framebuffer()
            .ok_or(format!("Failed to create frame buffer"))?;

        Ok(Self {
            ctx: ctx.clone(),
            viewport,
            handle,
            color_slot: (),
            depth_slot: (),
        })
    }
}

impl<D> Framebuffer<(), D> {
    pub fn with_color_slot<T>(self, ctx: &Ctx, tex: T) -> Framebuffer<T, D>
    where
        T: Deref<Target = UploadedTexture> + Clone,
    {
        self.bind();
        ctx.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::COLOR_ATTACHMENT0,
            GL::TEXTURE_2D,
            Some(&tex.handle),
            0,
        );

        Framebuffer {
            ctx: self.ctx,
            viewport: self.viewport,
            handle: self.handle,
            color_slot: tex.clone(),
            depth_slot: self.depth_slot,
        }
    }
}

impl<C> Framebuffer<C, ()> {
    pub fn with_depth_slot<T>(self, ctx: &Ctx, tex: T) -> Framebuffer<C, T>
    where
        T: Deref<Target = UploadedTexture> + Clone,
    {
        self.bind();
        ctx.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            GL::DEPTH_ATTACHMENT,
            GL::TEXTURE_2D,
            Some(&tex.handle),
            0,
        );

        Framebuffer {
            ctx: self.ctx,
            viewport: self.viewport,
            handle: self.handle,
            color_slot: self.color_slot, 
            depth_slot: tex.clone(),
        }
    }
}

impl<T, X> Framebuffer<T, X>
where
    T: Deref<Target = UploadedTexture> + Clone,
{
    pub fn color_slot(&self) -> T {
        self.color_slot.clone()
    }
}

impl<X, T> Framebuffer<X, T>
where
    T: Deref<Target = UploadedTexture> + Clone,
{
    pub fn depth_slot(&self) -> T {
        self.depth_slot.clone()
    }
}

// todo:
// impl<C, D> Drop for Framebuffer<C, D> {
//     fn drop(&mut self) {
//         self.ctx.delete_framebuffer(Some(&self.handle));
//     }
// }
