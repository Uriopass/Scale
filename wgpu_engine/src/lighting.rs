use crate::{compile_shader, Drawable, GfxContext, IndexType, Texture, Uniform, UvVertex, VBDesc};
use geom::LinearColor;
use mint::ColumnMatrix4;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{
    BlendFactor, Buffer, CommandEncoder, IndexFormat, MultisampleState, PrimitiveState, RenderPass,
    RenderPipeline, SwapChainFrame, TextureSampleType, VertexBufferLayout,
};

pub struct LightRender {
    noise: Texture,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    screen_vertex_buffer: Buffer,
}

impl LightRender {
    pub fn new(gfx: &mut GfxContext) -> Self {
        let noise = Texture::from_path(gfx, "assets/noise.png", None);

        let vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        let index_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(UV_INDICES),
            usage: wgpu::BufferUsage::INDEX,
        });

        let screen_vertex_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(SCREEN_UV_VERTICES),
            usage: wgpu::BufferUsage::VERTEX,
        });

        gfx.register_pipeline::<LightBlit>();
        gfx.register_pipeline::<LightMultiply>();

        Self {
            vertex_buffer,
            index_buffer,
            noise,
            screen_vertex_buffer,
        }
    }
}

#[derive(Copy, Clone)]
#[repr(C)]
struct LightUniform {
    inv_proj: ColumnMatrix4<f32>,
    ambiant: LinearColor,
    time: f32,
    height: f32,
}

u8slice_impl!(LightUniform);

struct LightBlit;

impl Drawable for LightBlit {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline
    where
        Self: Sized,
    {
        let vert_shader = compile_shader("assets/shaders/blit_light.vert", None);
        let frag_shader = compile_shader("assets/shaders/blit_light.frag", None);

        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("light pipeline"),
                    bind_group_layouts: &[&gfx.projection.layout],
                    push_constant_ranges: &[],
                });

        let vs_module = gfx.device.create_shader_module(&vert_shader.0);
        let fs_module = gfx.device.create_shader_module(&frag_shader.0);

        let color_states = [wgpu::ColorTargetState {
            format: gfx.light_texture.format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::One,
                    dst_factor: BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[UvVertex::desc(), LightInstance::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &color_states,
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };

        gfx.device.create_render_pipeline(&render_pipeline_desc)
    }

    fn draw<'a>(&'a self, _gfx: &'a GfxContext, _rp: &mut RenderPass<'a>) {
        unimplemented!()
    }
}

struct LightMultiply;
impl Drawable for LightMultiply {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline
    where
        Self: Sized,
    {
        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("basic pipeline"),
                    bind_group_layouts: &[
                        &Texture::bindgroup_layout_complex(
                            &gfx.device,
                            TextureSampleType::Float { filterable: true },
                            3,
                        ),
                        &Uniform::<LightUniform>::bindgroup_layout(&gfx.device),
                    ],
                    push_constant_ranges: &[],
                });

        let vs_module = gfx
            .device
            .create_shader_module(&compile_shader("assets/shaders/light_multiply.vert", None).0);
        let fs_module = gfx
            .device
            .create_shader_module(&compile_shader("assets/shaders/light_multiply.frag", None).0);

        let color_states = [wgpu::ColorTargetState {
            format: gfx.sc_desc.format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent::REPLACE,
                alpha: wgpu::BlendComponent::REPLACE,
            }),
            write_mask: wgpu::ColorWrite::ALL,
        }];

        let render_pipeline_desc = wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &vs_module,
                entry_point: "main",
                buffers: &[UvVertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &fs_module,
                entry_point: "main",
                targets: &color_states,
            }),
            primitive: PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
        };
        gfx.device.create_render_pipeline(&render_pipeline_desc)
    }

    fn draw<'a>(&'a self, _gfx: &'a GfxContext, _rp: &mut RenderPass<'a>) {
        unimplemented!()
    }
}

const UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-1.0, -1.0, 0.0],
        uv: [-1.0, 1.0],
    },
    UvVertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, -1.0],
    },
    UvVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [-1.0, -1.0],
    },
];

const SCREEN_UV_VERTICES: &[UvVertex] = &[
    UvVertex {
        position: [-1.0, -1.0, 0.0],
        uv: [0.0, 1.0],
    },
    UvVertex {
        position: [1.0, -1.0, 0.0],
        uv: [1.0, 1.0],
    },
    UvVertex {
        position: [1.0, 1.0, 0.0],
        uv: [1.0, 0.0],
    },
    UvVertex {
        position: [-1.0, 1.0, 0.0],
        uv: [0.0, 0.0],
    },
];

const UV_INDICES: &[IndexType] = &[0, 1, 2, 0, 2, 3];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct LightInstance {
    pub pos: [f32; 2],
    pub scale: f32,
}

u8slice_impl!(LightInstance);

impl VBDesc for LightInstance {
    fn desc<'a>() -> VertexBufferLayout<'a> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<LightInstance>() as wgpu::BufferAddress,
            step_mode: wgpu::InputStepMode::Instance,
            attributes: Box::leak(Box::new(
                wgpu::vertex_attr_array![2 => Float32x2, 3 => Float32],
            )),
        }
    }
}

impl LightRender {
    pub fn render_lights(
        &self,
        gfx: &GfxContext,
        encoder: &mut CommandEncoder,
        frame: &SwapChainFrame,
        lights: &[LightInstance],
        ambiant: LinearColor,
        height: f32,
    ) {
        let instance_buffer = gfx.device.create_buffer_init(&BufferInitDescriptor {
            label: None,
            contents: bytemuck::cast_slice(lights),
            usage: wgpu::BufferUsage::VERTEX,
        });

        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                    attachment: &gfx.light_texture.view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.0,
                            g: 0.0,
                            b: 0.0,
                            a: 1.0,
                        }),
                        store: true,
                    },
                }],
                depth_stencil_attachment: None,
            });
            rpass.set_pipeline(&gfx.get_pipeline::<LightBlit>());
            rpass.set_bind_group(0, &gfx.projection.bindgroup, &[]);
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, instance_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
            rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..lights.len() as u32);
        }

        let ambiant_uni = Uniform::new(
            LightUniform {
                inv_proj: *gfx.inv_projection.value(),
                time: *gfx.time_uni.value(),
                ambiant,
                height,
            },
            &gfx.device,
        );

        ambiant_uni.upload_to_gpu(&gfx.queue);

        let lmultiply_tex_bg = Texture::multi_bindgroup(
            &[&gfx.light_texture, &gfx.color_texture.target, &self.noise],
            &gfx.device,
            &gfx.get_pipeline::<LightMultiply>().get_bind_group_layout(0),
        );

        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachmentDescriptor {
                attachment: &frame.output.view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: true,
                },
            }],
            depth_stencil_attachment: None,
        });

        rpass.set_pipeline(&gfx.get_pipeline::<LightMultiply>());
        rpass.set_bind_group(0, &lmultiply_tex_bg, &[]);
        rpass.set_bind_group(1, &ambiant_uni.bindgroup, &[]);
        rpass.set_vertex_buffer(0, self.screen_vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.draw_indexed(0..UV_INDICES.len() as u32, 0, 0..1);
    }
}
