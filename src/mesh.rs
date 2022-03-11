use std::collections::HashMap;

use web_sys::WebGlBuffer;

use crate::{GL, Ctx, Program};
use crate::attributes::{Attribute, AttributeType};

struct VertexBuffer {
    ctx: Ctx,
    att: AttributeType,
    buffer: WebGlBuffer,
}

impl VertexBuffer {
    fn new<T: Attribute>(ctx: &Ctx, att: AttributeType, data: &T::Repr) -> Result<Self, String> {
        let buffer = ctx
            .create_buffer()
            .ok_or("Failed to create element buffer")?;
        ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&buffer));
        ctx.buffer_data_with_u8_array(GL::ARRAY_BUFFER, &T::pack(data), GL::STATIC_DRAW);

        Ok(Self {
            ctx: ctx.clone(),
            att,
            buffer,
        })
    }

    fn bind(&self, ptr_idx: u32) {
        self.ctx.bind_buffer(GL::ARRAY_BUFFER, Some(&self.buffer));
        self.ctx.vertex_attrib_pointer_with_i32(
            ptr_idx,
            self.att.num_components(),
            GL::FLOAT,
            false,
            0,
            0,
        );
    }
}

impl Drop for VertexBuffer {
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.buffer));
    }
}

struct ElementBuffer {
    ctx: Ctx,
    buffer: WebGlBuffer,
    num_elements: usize,
}

impl ElementBuffer {
    fn new(ctx: &Ctx, element_size_bytes: usize, data: &[u8]) -> Result<Self, String> {
        let buffer = ctx
            .create_buffer()
            .ok_or("Failed to create element buffer")?;
        ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&buffer));
        ctx.buffer_data_with_u8_array(GL::ELEMENT_ARRAY_BUFFER, data, GL::STATIC_DRAW);
        let num_elements = data.len() / element_size_bytes;

        Ok(Self {
            ctx: ctx.clone(),
            num_elements,
            buffer,
        })
    }

    fn draw(&self, mode: MeshMode) {
        self.ctx.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&self.buffer));
        self.ctx.draw_elements_with_i32(
            mode.0,
            self.num_elements as i32,
            GL::UNSIGNED_SHORT,
        0);
    }
}

impl Drop for ElementBuffer {
    fn drop(&mut self) {
        self.ctx.delete_buffer(Some(&self.buffer));
    }
}

#[derive(Clone, Copy)]
pub struct MeshMode(u32);

pub struct Mesh {
    ctx: Ctx,
    mode: MeshMode,
    vertex_buffers: HashMap<&'static str, VertexBuffer>,
    element_buffer: ElementBuffer,
}

impl Mesh {
    pub fn new(ctx: &Ctx, indices: &[u16]) -> Result<Self, String> {
        let data = indices.iter().flat_map(|e| e.to_ne_bytes()).collect::<Vec<u8>>();
        let eb = ElementBuffer::new(ctx, 2, &data)?;

        Ok(Self {
            ctx: ctx.clone(),
            mode: MeshMode(GL::TRIANGLES),
            vertex_buffers: HashMap::new(),
            element_buffer: eb
        })
    }

    pub fn with_attribute<T: Attribute>(mut self, name: &'static str, data: &T::Repr) -> Result<Self, String> {
        let vb = VertexBuffer::new::<T>(&self.ctx, T::new(name), data)?;
        self.vertex_buffers.insert(name, vb);
        Ok(self)
    }

    pub fn draw(&self, program: &Program) -> Result<(), String> {
        let mut enabled_attribs = vec![];
        for (&at, buf) in self.vertex_buffers.iter() {
            if let Some(idx) = Some(self.ctx.get_attrib_location(&program.program, at))
                .filter(|idx| *idx >= 0)
                .map(|idx| idx as u32) {
                    self.ctx.enable_vertex_attrib_array(idx as u32);
                    enabled_attribs.push(idx);
                    buf.bind(idx);
                }

        }
        self.element_buffer.draw(self.mode);

        for idx in enabled_attribs.into_iter() {
            self.ctx.disable_vertex_attrib_array(idx);
        }

        Ok(())
    }
}
