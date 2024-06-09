use std::iter;

use wgpu::util::DeviceExt;
use winit::{event::WindowEvent, window::Window};

use crate::{
    data_structures::Matrix,
    geometry::{Path, Point},
    render::{scale_to_fit, Renderable},
};

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    color: [f32; 3],
}

impl Vertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct ControlPointVertex {
    position: [f32; 3],
    uv: [f32; 3],
}

impl ControlPointVertex {
    fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<ControlPointVertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 0,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: std::mem::size_of::<[f32; 3]>() as wgpu::BufferAddress,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
            ],
        }
    }
}

fn curve_vertices(path: &Path, width: f32, height: f32) -> Vec<ControlPointVertex> {
    let mut vertices = Vec::new();

    for subpath in &path.subpaths {
        match subpath {
            crate::geometry::Subpath::Line(..) => {
                continue;
            }
            crate::geometry::Subpath::Quadratic(quad) => {
                vertices.push(ControlPointVertex {
                    position: [quad.start.x / width, quad.start.y / height, 0.0],
                    uv: [0.0, 0.0, 0.0],
                });
                vertices.push(ControlPointVertex {
                    position: [
                        quad.control_point.x / width,
                        quad.control_point.y / height,
                        0.0,
                    ],
                    uv: [0.5, 0.0, 0.0],
                });
                vertices.push(ControlPointVertex {
                    position: [quad.end.x / width, quad.end.y / height, 0.0],
                    uv: [1.0, 1.0, 0.0],
                });
            }
            crate::geometry::Subpath::Cubic(..) => {
                // todo!()
            }
        }
    }

    vertices
}

#[allow(unused)]
fn compile_cubic(start: Point, c0: Point, c1: Point, end: Point) -> () {}

fn vertices_for_path(path: &Path, width: f32, height: f32, color: u32) -> Vec<Vec<Vertex>> {
    let point_to_vertex = |p: Point| -> Vertex {
        let b = ((color >> 16) & 0xff) as f32 / 255.0;
        let g = ((color >> 8) & 0xff) as f32 / 255.0;
        let r = ((color >> 0) & 0xff) as f32 / 255.0;

        Vertex {
            position: [p.x / width, p.y / height, 0.0],
            color: [r, g, b],
        }
    };

    let mut out = Vec::new();

    let mut vertices = Vec::new();

    let mut last_point = path.subpaths[0].start();
    for subpath in &path.subpaths {
        if subpath.start() != last_point {
            out.push(vertices);
            vertices = Vec::new();
        }

        match subpath {
            crate::geometry::Subpath::Line(line) => {
                if vertices.is_empty() {
                    vertices.push(point_to_vertex(line.start));
                }
                vertices.push(point_to_vertex(line.end));
            }
            crate::geometry::Subpath::Quadratic(quad) => {
                if vertices.is_empty() {
                    vertices.push(point_to_vertex(quad.start));
                }
                vertices.push(point_to_vertex(quad.end));
            }
            crate::geometry::Subpath::Cubic(cub) => {
                if vertices.is_empty() {
                    vertices.push(point_to_vertex(cub.start));
                }
                vertices.push(point_to_vertex(cub.end));
            }
        }
        last_point = subpath.end();
    }

    out.push(vertices);

    out
}

fn indices_for_outline(num_vertices: usize, offset: u32) -> Vec<u32> {
    let mut indices = Vec::new();

    for i in 0..(num_vertices as u32 - 1) {
        indices.push(0 + offset);
        indices.push(i + offset);
        indices.push(i + 1 + offset);
    }

    indices
}

// We need this for Rust to store our data correctly for the shaders
#[repr(C)]
// This is so we can store this in a buffer
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    // We can't use cgmath with bytemuck directly, so we'll have
    // to convert the Matrix4 into a 4x4 f32 array
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new(transform: Matrix) -> Self {
        Self {
            view_proj: transform.into(),
        }
    }
}

struct CachedBuffers {
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    curve_vertex_buffer: wgpu::Buffer,
    vertex_buffer_render: wgpu::Buffer,
    vertex_buffer_render_len: u32,
}

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: wgpu::TextureView,
}

pub(super) struct State<'a> {
    surface: wgpu::Surface<'a>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub(super) size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    camera_bind_group_layout: wgpu::BindGroupLayout,
    mask_pipeline: wgpu::RenderPipeline,
    curve_mask_pipeline: wgpu::RenderPipeline,
    window: &'a Window,
    cached_buffers: Option<CachedBuffers>,
}

impl<'a> State<'a> {
    fn init_buffers(&mut self, to_render: &[Renderable], width: f32, height: f32) {
        if self.cached_buffers.is_some() {
            return;
        }

        let mut indices = Vec::new();
        let mut vertices = Vec::new();

        for r in to_render {
            for p in &r.outline.paths {
                let paths = vertices_for_path(
                    p,
                    width,
                    height,
                    r.fill_color.unwrap_or_else(|| r.stroke_color.unwrap_or(0)),
                );
                for mut path in paths {
                    indices.append(&mut indices_for_outline(path.len(), vertices.len() as u32));
                    vertices.append(&mut path);
                }
            }
        }
        let vertex_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Vertex Buffer"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
        let index_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Index Buffer"),
                contents: bytemuck::cast_slice(&indices),
                usage: wgpu::BufferUsages::INDEX,
            });

        let num_indices = indices.len() as u32;

        let mut vertices_on_curve = Vec::new();
        for r in to_render {
            for p in &r.outline.paths {
                vertices_on_curve.append(&mut curve_vertices(p, width, height));
            }
        }

        let curve_vertex_buffer =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("curve Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices_on_curve),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        let mut vertices_render = Vec::new();

        for idx in &indices {
            vertices_render.push(vertices[*idx as usize]);
        }

        for vertex in vertices_on_curve {
            vertices_render.push(Vertex {
                position: vertex.position,
                color: [0.0, 0.0, 0.0],
            });
        }

        let vertex_buffer_render =
            self.device
                .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Vertex Buffer"),
                    contents: bytemuck::cast_slice(&vertices_render),
                    usage: wgpu::BufferUsages::VERTEX,
                });

        self.cached_buffers = Some(CachedBuffers {
            vertex_buffer,
            index_buffer,
            curve_vertex_buffer,
            vertex_buffer_render,
            num_indices,
            vertex_buffer_render_len: vertices_render.len() as u32,
        });
    }

    pub async fn new(window: &'a Window) -> State<'a> {
        let size = window.inner_size();

        // The instance is a handle to our GPU
        // BackendBit::PRIMARY => Vulkan + Metal + DX12 + Browser WebGPU
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        // # Safety
        //
        // The surface needs to live as long as the window that created it.
        // State owns the window so this should be safe.
        let surface = instance.create_surface(window).unwrap();

        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // WebGL doesn't support all of wgpu's features, so if
                    // we're building for the web we'll have to disable some.
                    required_limits: if cfg!(target_arch = "wasm32") {
                        wgpu::Limits::downlevel_webgl2_defaults()
                    } else {
                        wgpu::Limits::default()
                    },
                },
                None, // Trace path
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        // Shader code in this tutorial assumes an Srgb surface texture. Using a different
        // one will result all the colors comming out darker. If you want to support non
        // Srgb surfaces, you'll need to account for that when drawing to the frame.
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout],
                push_constant_ranges: &[],
            });

        let stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Equal,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Keep,
        };

        let primitive = wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            // ohohoh!?
            cull_mode: None,
            // Setting this to anything other than Fill requires Features::POLYGON_MODE_LINE
            // or Features::POLYGON_MODE_POINT
            polygon_mode: wgpu::PolygonMode::Fill,
            // Requires Features::DEPTH_CLIP_CONTROL
            unclipped_depth: false,
            // Requires Features::CONSERVATIVE_RASTERIZATION
            conservative: false,
        };

        let mask_stencil_state = wgpu::StencilFaceState {
            compare: wgpu::CompareFunction::Always,
            fail_op: wgpu::StencilOperation::Keep,
            depth_fail_op: wgpu::StencilOperation::Keep,
            pass_op: wgpu::StencilOperation::Invert,
        };

        let depth_stencil = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Stencil8,
            depth_write_enabled: false,
            depth_compare: wgpu::CompareFunction::Always,
            bias: wgpu::DepthBiasState::default(),
            stencil: wgpu::StencilState {
                front: mask_stencil_state,
                back: mask_stencil_state,
                // Applied to values being read from the buffer
                read_mask: 0xff,
                // Applied to values before being written to the buffer
                write_mask: 0xff,
            },
        };

        let mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("mask Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: None,
            primitive,
            depth_stencil: Some(depth_stencil.clone()),

            multisample: wgpu::MultisampleState::default(),
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let curve_mask_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("curve mask Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "curve_fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive,
            depth_stencil: Some(depth_stencil),

            multisample: wgpu::MultisampleState::default(),
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[Vertex::desc()],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState {
                        color: wgpu::BlendComponent::OVER,
                        alpha: wgpu::BlendComponent::OVER,
                    }),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive,
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Stencil8,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Always,
                bias: wgpu::DepthBiasState::default(),
                stencil: wgpu::StencilState {
                    front: stencil_state,
                    back: stencil_state,
                    // Applied to values being read from the buffer
                    read_mask: 0xff,
                    // Applied to values before being written to the buffer
                    write_mask: 0xff,
                },
            }),

            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            // If the pipeline will be used with a multiview render pass, this
            // indicates how many array layers the attachments will have.
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            mask_pipeline,
            curve_mask_pipeline,
            window,
            camera_bind_group_layout,
            cached_buffers: None,
        }
    }

    pub fn window(&self) -> &Window {
        &self.window
    }

    pub fn resize(&mut self, mut new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            let scale = scale_to_fit(new_size.width as f32, new_size.height as f32);
            new_size = winit::dpi::PhysicalSize::new(
                (new_size.width as f32 * scale) as u32,
                (new_size.height as f32 * scale) as u32,
            );
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
        }
    }

    #[allow(unused_variables)]
    pub fn input(&mut self, event: &WindowEvent) -> bool {
        false
    }

    pub fn update(&mut self) {}

    fn create_stencil_texture(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
        label: &str,
    ) -> Texture {
        let size = wgpu::Extent3d {
            // 2.
            width: config.width,
            height: config.height,
            depth_or_array_layers: 1,
        };
        let desc = wgpu::TextureDescriptor {
            label: Some(label),
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Stencil8,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT // 3.
                | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Texture { texture, view }
    }

    fn encoder(&self, label: &'static str) -> wgpu::CommandEncoder {
        self.device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) })
    }

    pub fn render(
        &mut self,
        to_render: &[Renderable],
        width: f32,
        height: f32,
        transform: Matrix,
    ) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let stencil_texture =
            Self::create_stencil_texture(&self.device, &self.config, "stencil_texture");

        let camera_uniform = CameraUniform::new(transform);
        let camera_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Camera Buffer"),
                contents: bytemuck::cast_slice(&[camera_uniform]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let camera_bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &self.camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let mut mask_encoder = self.encoder("mask encoder");
        let mut curve_mask_encoder = self.encoder("curve mask encoder");
        let mut encoder = self.encoder("render encoder");

        self.init_buffers(to_render, width, height);
        let CachedBuffers {
            vertex_buffer,
            index_buffer,
            num_indices,
            curve_vertex_buffer,
            vertex_buffer_render,
            vertex_buffer_render_len,
        } = self.cached_buffers.as_ref().unwrap();

        {
            let mut mask_pass: wgpu::RenderPass =
                mask_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("mask Pass"),
                    color_attachments: &[],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &stencil_texture.view,
                        depth_ops: None,
                        stencil_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            mask_pass.set_pipeline(&self.mask_pipeline);
            mask_pass.set_bind_group(0, &camera_bind_group, &[]);
            mask_pass.set_vertex_buffer(0, vertex_buffer.slice(..));
            mask_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            mask_pass.draw_indexed(0..*num_indices, 0, 0..1);
        }

        self.queue.submit(iter::once(mask_encoder.finish()));

        let mut vertices_on_curve = Vec::new();
        for r in to_render {
            for p in &r.outline.paths {
                vertices_on_curve.append(&mut curve_vertices(p, width, height));
            }
        }

        {
            let mut curve_mask_pass: wgpu::RenderPass =
                curve_mask_encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("curve mask Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &stencil_texture.view,
                        depth_ops: None,
                        stencil_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Load,
                            store: wgpu::StoreOp::Store,
                        }),
                    }),
                    occlusion_query_set: None,
                    timestamp_writes: None,
                });

            curve_mask_pass.set_pipeline(&self.curve_mask_pipeline);
            curve_mask_pass.set_bind_group(0, &camera_bind_group, &[]);
            curve_mask_pass.set_vertex_buffer(0, curve_vertex_buffer.slice(..));
            curve_mask_pass.draw(0..vertices_on_curve.len() as u32, 0..1);
        }

        self.queue.submit(iter::once(curve_mask_encoder.finish()));

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    stencil_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    }),
                    view: &stencil_texture.view,
                    depth_ops: None,
                }),

                occlusion_query_set: None,
                timestamp_writes: None,
            });

            static mut FRAME: u32 = 1;

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &camera_bind_group, &[]);
            render_pass.set_vertex_buffer(0, vertex_buffer_render.slice(..));
            // render_pass.set_index_buffer(index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            render_pass.set_stencil_reference(0xff);
            render_pass.draw(0..*vertex_buffer_render_len, 0..1);
            // render_pass.draw_indexed(0..vertices_render.len() as u32, 0, 0..1);
            // dbg!(unsafe { FRAME });
            unsafe { FRAME += 1 };
        }

        self.queue.submit(iter::once(encoder.finish()));
        output.present();

        // std::thread::sleep(std::time::Duration::from_millis(500));

        Ok(())
    }
}
