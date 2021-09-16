use std::collections::HashMap;

use ike::prelude::*;

use crate::game_state::GameState;

#[derive(Default)]
pub struct D3Pass {
    color: Option<wgpu::TextureView>,
    depth: Option<wgpu::TextureView>,
}

impl RenderPass<GameState> for D3Pass {
    fn run<'a>(
        &'a mut self,
        encoder: &'a mut wgpu::CommandEncoder,
        ctx: &RenderCtx,
        view: &'a View,
        data: &mut PassData,
        state: &mut GameState,
    ) -> wgpu::RenderPass<'a> {
        data.insert(SampleCount(1));
        data.insert(TargetFormat(state.d3_buffer.descriptor.format));
        data.insert(TargetSize {
            width: state.d3_buffer.descriptor.width,
            height: state.d3_buffer.descriptor.height,
        });
        data.insert(view.camera.clone());

        let height = state.config.graphics.d3_scale;

        let aspect = view.width as f32 / view.height as f32;

        let width = (aspect * height as f32).floor() as u32;

        state.d3_buffer.descriptor.width = width;
        state.d3_buffer.descriptor.height = height;
        state.d3_buffer.descriptor.usage |= wgpu::TextureUsages::TEXTURE_BINDING;

        let (color, depth) = state.d3_buffer.color_depth(ctx);

        self.color = Some(color.create_view(&Default::default()));
        self.depth = Some(depth.create_view(&Default::default()));

        encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: self.color.as_ref().unwrap(),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: true,
                },
            }],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: self.depth.as_ref().unwrap(),
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: true,
                }),
                stencil_ops: None,
            }),
        })
    }
}

#[derive(Default)]
pub struct RenderNode {
    shader_module: Option<wgpu::ShaderModule>,
    bind_group_version: u64,
    bind_group: Option<wgpu::BindGroup>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
}

impl RenderNode {
    #[inline]
    fn create_bind_group(&mut self, render_ctx: &RenderCtx, state: &mut GameState) {
        let bind_group_layout =
            render_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            ty: wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                    ],
                });

        let (color, depth) = state.d3_buffer.color_depth(render_ctx);

        let color = color.create_view(&Default::default());
        let depth = depth.create_view(&Default::default());

        let sampler = render_ctx.device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        self.bind_group = Some(
            render_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: &bind_group_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::TextureView(&color),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(&depth),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: wgpu::BindingResource::Sampler(&sampler),
                        },
                    ],
                }),
        );
    }

    #[inline]
    fn create_pipeline_layout(&mut self, render_ctx: &RenderCtx) {
        self.shader_module = Some(
            render_ctx
                .device
                .create_shader_module(&wgpu::include_wgsl!("combine.wgsl")),
        );

        let bind_group_layout =
            render_ctx
                .device
                .create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                    label: None,
                    entries: &[
                        wgpu::BindGroupLayoutEntry {
                            binding: 0,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 1,
                            ty: wgpu::BindingType::Texture {
                                sample_type: wgpu::TextureSampleType::Depth,
                                view_dimension: wgpu::TextureViewDimension::D2,
                                multisampled: false,
                            },
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                        wgpu::BindGroupLayoutEntry {
                            binding: 2,
                            ty: wgpu::BindingType::Sampler {
                                filtering: true,
                                comparison: false,
                            },
                            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                            count: None,
                        },
                    ],
                });

        self.pipeline_layout = Some(render_ctx.device.create_pipeline_layout(
            &wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&bind_group_layout],
                push_constant_ranges: &[],
            },
        ));
    }

    #[inline]
    fn create_pipeline(
        &mut self,
        render_ctx: &RenderCtx,
        format: wgpu::TextureFormat,
        samples: u32,
    ) -> wgpu::RenderPipeline {
        render_ctx
            .device
            .create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: None,
                layout: Some(self.pipeline_layout.as_ref().unwrap()),
                vertex: wgpu::VertexState {
                    module: self.shader_module.as_ref().unwrap(),
                    entry_point: "main",
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: self.shader_module.as_ref().unwrap(),
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: Default::default(),
                multisample: wgpu::MultisampleState {
                    count: samples,
                    ..Default::default()
                },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
            })
    }
}

impl PassNode<GameState> for RenderNode {
    #[inline]
    fn run<'a>(&'a mut self, ctx: &mut PassNodeCtx<'_, 'a>, state: &mut GameState) {
        if self.bind_group.is_none() || self.bind_group_version != state.d3_buffer.version() {
            self.bind_group_version = state.d3_buffer.version();

            self.create_bind_group(ctx.render_ctx, state);
        }

        if self.pipeline_layout.is_none() {
            self.create_pipeline_layout(ctx.render_ctx);
        }

        if !self.pipelines.contains_key(&ctx.view.format) {
            let pipeline = self.create_pipeline(
                ctx.render_ctx,
                ctx.view.format,
                ctx.data.get::<SampleCount>().unwrap().0,
            );
            self.pipelines.insert(ctx.view.format, pipeline);
        }

        let pipeline = self.pipelines.get(&ctx.view.format).unwrap();

        ctx.render_pass.set_pipeline(pipeline);

        ctx.render_pass
            .set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        ctx.render_pass.draw(0..3, 0..1);
    }
}
