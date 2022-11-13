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

    pub fn dimensions(&self) -> [f32; 2] { [self.w as f32, self.h as f32] }

    pub fn set(&mut self, ctx: &Ctx) {
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

impl From<ColorFormat> for i32 {
    fn from(v: ColorFormat) -> Self {
        v.0 as i32
    }
}

impl From<ColorFormat> for u32 {
    fn from(v: ColorFormat) -> Self {
        v.0
    }
}

#[derive(Clone, Copy)]
pub struct InterpolationMin(pub u32);

impl From<InterpolationMin> for i32 {
    fn from(v: InterpolationMin) -> Self {
        v.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct InterpolationMag(pub u32);

impl From<InterpolationMag> for i32 {
    fn from(v: InterpolationMag) -> Self {
        v.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct WrapT(pub u32);

impl From<WrapT> for i32 {
    fn from(v: WrapT) -> Self {
        v.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct WrapS(pub u32);


impl From<WrapS> for i32 {
    fn from(v: WrapS) -> Self {
        v.0 as i32
    }
}

#[derive(Clone, Copy)]
pub struct InternalFormat(pub u32);

impl From<InternalFormat> for u32 {
    fn from(v: InternalFormat) -> Self {
        v.0 
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

    pub fn depth(dimensions: [u32; 2]) -> Self {
        Self {
            color_format: ColorFormat(GL::DEPTH_COMPONENT),
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

    pub fn upload_u8(&self, ctx: &Ctx, data: &[u8]) -> Result<UploadedTexture, String> {
        let arr = js_sys::Uint8Array::new_with_length(data.len() as u32);
        arr.copy_from(data);
        self.upload(ctx, InternalFormat(GL::UNSIGNED_BYTE), Some(&arr))
    }

    pub fn upload_rgba(&self, ctx: &Ctx, data: &[[f32; 4]]) -> Result<UploadedTexture, String> {
        self.upload_f32(ctx, &data.iter().flat_map(|v| v.to_vec()).collect::<Vec<f32>>())
    }

    pub fn upload_f32(&self, ctx: &Ctx, data: &[f32]) -> Result<UploadedTexture, String> {
        let arr = js_sys::Float32Array::new_with_length(data.len() as u32);
        arr.copy_from(data);
        self.upload(ctx, InternalFormat(GL::FLOAT), Some(&arr))
    }

    pub fn upload(&self, ctx: &Ctx, internal_format: InternalFormat, data: Option<&js_sys::Object>) -> Result<UploadedTexture, String> {
        let handle = ctx
            .create_texture()
            .ok_or("Failed to create texture")?;
        ctx.bind_texture(GL::TEXTURE_2D, Some(&handle));
        ctx.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
            GL::TEXTURE_2D,
            0,
            self.color_format.into(),
            self.dimensions[0] as i32,
            self.dimensions[1] as i32,
            0,
            self.color_format.into(),
            internal_format.into(),
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
                    size: self.dimensions,
            }
        )
    }
}

pub struct UploadedTexture {
    ctx: Ctx,
    handle: WebGlTexture,
    size: [u32; 2],
}

impl UploadedTexture {
    pub fn bind(&mut self) {
        self.ctx.bind_texture(GL::TEXTURE_2D, Some(&self.handle));
    }

    pub fn sizef32(&self) -> [f32; 2] {
        [self.size[0] as f32, self.size[1] as f32]
    }
}

impl Drop for UploadedTexture {
    fn drop(&mut self) {
        self.ctx.delete_texture(Some(&self.handle));
    }
}

enum FramebufferSlot {
    Color,
    Depth,
}

impl From<FramebufferSlot> for u32 {
    fn from(slot: FramebufferSlot) -> Self {
        match slot {
            FramebufferSlot::Color => GL::COLOR_ATTACHMENT0,
            FramebufferSlot::Depth => GL::DEPTH_ATTACHMENT,
        }
    }
}

pub trait Framebuffer {
    type DepthSlot;
    type ColorSlot;

    fn bind(&mut self);
    fn depth_slot(&mut self) -> &mut Self::DepthSlot;
    fn color_slot(&mut self) -> &mut Self::ColorSlot;
    fn viewport(&self) -> &Viewport;
}

pub struct EmptyFramebuffer {
    ctx: Ctx,
    viewport: Viewport,
}

impl EmptyFramebuffer {
    pub fn new(ctx: &Ctx, viewport: Viewport) -> Self {
         Self {
             ctx: ctx.clone(),
             viewport,
         }
    }

    fn bind(&mut self) {
        self.ctx.bind_framebuffer(GL::FRAMEBUFFER, None);
        self.viewport.set(&self.ctx);
    }

    pub fn with_color_slot(self, handle: UploadedTexture) -> Result<ColorFramebuffer, String> {
        Ok(ColorFramebuffer { fb: FramebufferWithSlot::from_fb(self, FramebufferSlot::Color, handle)? })
    }

    pub fn with_depth_slot(self, handle: UploadedTexture) -> Result<DepthFrameBuffer, String> {
        Ok(DepthFrameBuffer { fb: FramebufferWithSlot::from_fb(self, FramebufferSlot::Depth, handle)? })
    }
}

struct FramebufferWithSlot {
    ctx: Ctx,
    viewport: Viewport,
    handle: WebGlFramebuffer,
    slot: UploadedTexture,
}

impl FramebufferWithSlot {
    fn from_fb(fb: EmptyFramebuffer, attachment: FramebufferSlot, handle: UploadedTexture) -> Result<FramebufferWithSlot, String> {
         let fb_handle = fb.ctx
             .create_framebuffer()
             .ok_or("Failed to create frame buffer")?;

        let mut result = Self {
            ctx: fb.ctx,
            viewport: fb.viewport,
            handle: fb_handle,
            slot: handle,
        };

        result.ctx.bind_texture(GL::TEXTURE_2D, None);
        result.bind();
        result.ctx.framebuffer_texture_2d(
            GL::FRAMEBUFFER,
            attachment.into(),
            GL::TEXTURE_2D,
            Some(&result.slot.handle),
            0,
        );

        Ok(result)
    }

    fn bind(&mut self) {
        self.ctx.bind_texture(GL::TEXTURE_2D, None);
        self.ctx.bind_framebuffer(GL::FRAMEBUFFER, Some(&self.handle));
        self.viewport.set(&self.ctx);
    }
}

impl Framebuffer for EmptyFramebuffer {
    type DepthSlot = Self;
    type ColorSlot = Self;

    fn depth_slot(&mut self) -> &mut Self { self }
    fn color_slot(&mut self) -> &mut Self { self }

    fn bind(&mut self) {
        self.bind()
    }

    fn viewport(&self) -> &Viewport { &self.viewport }
}

pub struct ColorFramebuffer {
    fb: FramebufferWithSlot,
}

impl Framebuffer for ColorFramebuffer {
    type DepthSlot = Self;
    type ColorSlot = UploadedTexture;

    fn depth_slot(&mut self) -> &mut Self::DepthSlot { self }
    fn color_slot(&mut self) -> &mut Self::ColorSlot { &mut self.fb.slot }

    fn bind(&mut self) {
        self.fb.bind()
    }

    fn viewport(&self) -> &Viewport { &self.fb.viewport }
}

pub struct DepthFrameBuffer {
    fb: FramebufferWithSlot,
}

impl Framebuffer for DepthFrameBuffer {
    type DepthSlot = UploadedTexture;
    type ColorSlot = Self;

    fn depth_slot(&mut self) -> &mut Self::DepthSlot { &mut self.fb.slot }
    fn color_slot(&mut self) -> &mut Self::ColorSlot { self }

    fn bind(&mut self) {
        self.fb.bind()
    }

    fn viewport(&self) -> &Viewport { &self.fb.viewport }
}

impl Drop for FramebufferWithSlot {
    fn drop(&mut self) {
        self.ctx.delete_framebuffer(Some(&self.handle));
    }
}
