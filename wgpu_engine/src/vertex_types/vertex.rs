use crate::VBDesc;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColoredVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
}

u8slice_impl!(ColoredVertex);

impl VBDesc for ColoredVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        use std::mem;
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<ColoredVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: Box::leak(Box::new(
                wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4],
            )),
        }
    }
}
