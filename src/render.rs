use std::collections::HashMap;

use bytemuck::{bytes_of, cast_slice};
pub use ike::prelude::*;
use ike::wgpu::util::DeviceExt;
use ike::d3::mesh::{Vertices, Indices};

use crate::game_state::GameState;

struct MeshInstance {
    index_count: u32,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    instances: Vec<Mat4>,
    instance_buffer: Option<wgpu::Buffer>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MeshId {
    pub vertex: Id<Vertices>,
    pub index: Id<Indices>,
}

impl MeshId {
    #[inline]
    pub fn new(mesh: &Mesh) -> Self {
        Self {
            vertex: mesh.id(),
            index: mesh.id(),
        }
    }
}

pub struct Ctx<'a> {
    render_ctx: &'a RenderCtx, 
    meshes: &'a mut HashMap<MeshId, MeshInstance>,
}

impl<'a> Ctx<'a> {
    #[inline]
    pub fn render_mesh(&mut self, mesh: &Mesh, transform: Mat4) {
        let id = MeshId::new(mesh);

        if let Some(instance) = self.meshes.get_mut(&id) {
            let data = mesh.data();

            if mesh.vertices.mutated() { 
                let vertex_buffer =
                    self.render_ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: cast_slice(&data.vertex_data), 
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                        }); 

                instance.vertex_buffer = vertex_buffer; 

                mesh.vertices.reset_mutated();
            }

            if mesh.indices.mutated() {
                let index_buffer =
                    self.render_ctx
                        .device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: None,
                            contents: cast_slice(&data.index_data),
                            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
                        });

                instance.index_buffer = index_buffer;
                instance.index_count = data.index_count; 

                mesh.indices.reset_mutated();
            }

            instance.instances.push(transform);
        } else {
            let data = mesh.data();

            let vertex_buffer =
                self.render_ctx
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: cast_slice(&data.vertex_data),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                    });

            let index_buffer =
                self.render_ctx
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: cast_slice(&data.index_data),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::INDEX,
                    });

            let instance = MeshInstance {
                index_count: data.index_count,
                vertex_buffer,
                index_buffer,
                instances: vec![transform],
                instance_buffer: None,
            };

            self.meshes.insert(id, instance);

            mesh.vertices.reset_mutated();
            mesh.indices.reset_mutated();
        }
    }
}

#[derive(Default)]
pub struct RenderNode {
    width: u32,
    height: u32,
    texture: Option<wgpu::TextureView>,
    depth: Option<wgpu::TextureView>,
    shader_module: Option<wgpu::ShaderModule>,
    bind_group: Option<wgpu::BindGroup>,
    pipeline_layout: Option<wgpu::PipelineLayout>,
    uniforms_buffer: Option<wgpu::Buffer>,
    uniforms_bind_group_layout: Option<wgpu::BindGroupLayout>,
    uniforms_bind_group: Option<wgpu::BindGroup>,
    pipeline: Option<wgpu::RenderPipeline>,
    pipelines: HashMap<wgpu::TextureFormat, wgpu::RenderPipeline>,
    meshes: HashMap<MeshId, MeshInstance>,
}

impl RenderNode {
    #[inline]
    fn create_textures(&mut self, render_ctx: &RenderCtx) {
        let texture = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        let depth = render_ctx.device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: wgpu::Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth24Plus,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        });

        self.texture = Some(texture.create_view(&Default::default()));
        self.depth = Some(depth.create_view(&Default::default()));

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
                            resource: wgpu::BindingResource::TextureView(
                                self.texture.as_ref().unwrap(),
                            ),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: wgpu::BindingResource::TextureView(
                                self.depth.as_ref().unwrap(),
                            ),
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

        self.uniforms_bind_group_layout = Some(render_ctx.device.create_bind_group_layout(
            &wgpu::BindGroupLayoutDescriptor {
                label: Some("render_node_uniforms"),
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    count: None,
                }],
            },
        ));

        let layout = render_ctx
            .device
            .create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("render_node_pipeline_layout"),
                bind_group_layouts: &[self.uniforms_bind_group_layout.as_ref().unwrap()],
                push_constant_ranges: &[],
            });

        let shader_module = render_ctx
            .device
            .create_shader_module(&wgpu::include_wgsl!("shader.wgsl"));

        self.pipeline = Some(render_ctx.device.create_render_pipeline(
            &wgpu::RenderPipelineDescriptor {
                label: Some("render_node_pipeline"),
                layout: Some(&layout),
                vertex: wgpu::VertexState {
                    module: &shader_module,
                    entry_point: "main",
                    buffers: &[
                        wgpu::VertexBufferLayout {
                            array_stride: 48,
                            step_mode: wgpu::VertexStepMode::Vertex,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 0,
                                    shader_location: 0,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x3,
                                    offset: 12,
                                    shader_location: 1,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x2,
                                    offset: 24,
                                    shader_location: 2,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 32,
                                    shader_location: 3,
                                },
                            ],
                        },
                        wgpu::VertexBufferLayout {
                            array_stride: 64,
                            step_mode: wgpu::VertexStepMode::Instance,
                            attributes: &[
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 0,
                                    shader_location: 4,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 16,
                                    shader_location: 5,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 32,
                                    shader_location: 6,
                                },
                                wgpu::VertexAttribute {
                                    format: wgpu::VertexFormat::Float32x4,
                                    offset: 48,
                                    shader_location: 7,
                                },
                            ],
                        },
                    ],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &shader_module,
                    entry_point: "main",
                    targets: &[wgpu::ColorTargetState {
                        format: wgpu::TextureFormat::Rgba8UnormSrgb,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    }],
                }),
                primitive: wgpu::PrimitiveState::default(),
                multisample: wgpu::MultisampleState::default(),
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: wgpu::TextureFormat::Depth24Plus,
                    depth_write_enabled: true,
                    depth_compare: wgpu::CompareFunction::LessEqual,
                    stencil: Default::default(),
                    bias: Default::default(),
                }),
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
        let height = state.config.graphics.d3_scale;

        let aspect = ctx.view.width as f32 / ctx.view.height as f32;

        let width = (aspect * height as f32).floor() as u32;

        if self.width != width || self.height != height {
            self.width = width;
            self.height = height;

            self.create_textures(ctx.render_ctx);
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

        {
            let mut ctx = Ctx {
                render_ctx: ctx.render_ctx,
                meshes: &mut self.meshes,
            };

            state.render(&mut ctx);
        }

        let uniforms_buffer = if let Some(ref buffer) = self.uniforms_buffer {
            ctx.render_ctx
                .queue
                .write_buffer(buffer, 0, bytes_of(&ctx.view.view_proj));

            buffer
        } else {
            let buffer =
                ctx.render_ctx
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: None,
                        contents: bytes_of(&ctx.view.view_proj),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::UNIFORM,
                    });

            self.uniforms_buffer = Some(buffer);

            self.uniforms_buffer.as_ref().unwrap()
        };

        if self.uniforms_bind_group.is_none() {
            let bind_group = ctx
                .render_ctx
                .device
                .create_bind_group(&wgpu::BindGroupDescriptor {
                    label: None,
                    layout: self.uniforms_bind_group_layout.as_ref().unwrap(),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniforms_buffer.as_entire_binding(),
                    }],
                });

            self.uniforms_bind_group = Some(bind_group);
        }

        for (_id, mesh) in &mut self.meshes {
            let instance_buffer =
                ctx.render_ctx
                    .device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("render_node_instance_buffer"),
                        contents: cast_slice(&mesh.instances),
                        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::VERTEX,
                    });

            mesh.instance_buffer = Some(instance_buffer);
        }

        let mut encoder = ctx
            .render_ctx
            .device
            .create_command_encoder(&Default::default());

        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[wgpu::RenderPassColorAttachment {
                view: self.texture.as_ref().unwrap(),
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
        });

        render_pass.set_pipeline(self.pipeline.as_ref().unwrap());

        for (_id, mesh) in &mut self.meshes {
            render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            render_pass.set_vertex_buffer(1, mesh.instance_buffer.as_ref().unwrap().slice(..));
            render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

            render_pass.set_bind_group(0, self.uniforms_bind_group.as_ref().unwrap(), &[]);

            render_pass.draw_indexed(0..mesh.index_count, 0, 0..mesh.instances.len() as u32);

            mesh.instances.clear();
        }

        drop(render_pass);

        ctx.render_ctx
            .queue
            .submit(std::iter::once(encoder.finish()));

        ctx.render_pass.set_pipeline(pipeline);

        ctx.render_pass
            .set_bind_group(0, self.bind_group.as_ref().unwrap(), &[]);

        ctx.render_pass.draw(0..3, 0..1);
    }
}
