use crate::{compile_shader, Drawable, GfxContext, Texture, UvVertex, VBDesc};
use wgpu::{
    BlendFactor, BlendOperation, MultisampleState, PrimitiveState, RenderPass, RenderPipeline,
};

pub struct BlitLinear;

impl Drawable for BlitLinear {
    fn create_pipeline(gfx: &GfxContext) -> RenderPipeline
    where
        Self: Sized,
    {
        let render_pipeline_layout =
            gfx.device
                .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                    label: Some("blit pipeline"),
                    bind_group_layouts: &[&Texture::bindgroup_layout(&gfx.device)],
                    push_constant_ranges: &[],
                });

        let vs_module = gfx
            .device
            .create_shader_module(&compile_shader("assets/shaders/blit_linear.vert", None).0);
        let fs_module = gfx
            .device
            .create_shader_module(&compile_shader("assets/shaders/blit_linear.frag", None).0);

        let color_states = [wgpu::ColorTargetState {
            format: gfx.sc_desc.format,
            blend: Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: BlendFactor::SrcAlpha,
                    dst_factor: BlendFactor::OneMinusSrcAlpha,
                    operation: BlendOperation::Add,
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
