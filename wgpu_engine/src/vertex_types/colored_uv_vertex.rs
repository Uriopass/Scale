use crate::VBDesc;

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct ColoredUvVertex {
    pub position: [f32; 3],
    pub color: [f32; 4],
    pub uv: [f32; 2],
}

u8slice_impl!(ColoredUvVertex);

impl VBDesc for ColoredUvVertex {
    fn desc<'a>() -> wgpu::VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ColoredUvVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Vertex,
            attributes: Box::leak(Box::new(
                wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2],
            )),
        }
    }
}
