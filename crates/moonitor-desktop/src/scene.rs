//! A small native mesh renderer. Geometry is uploaded once; only the camera and
//! selection uniforms change. No game loop, asset service, or web runtime.
use crate::{config::VisualKind, palette, ui::Message, view_model::DeviceSceneModel};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use iced::wgpu;
use iced::{Rectangle, mouse, widget::shader};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use wgpu::util::DeviceExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub yaw: f32,
    pub zoom: f32,
}
impl Default for Camera {
    fn default() -> Self {
        Self {
            yaw: 0.30,
            zoom: 1.0,
        }
    }
}
fn camera(camera: Camera, aspect: f32, devices: &[DeviceSceneModel]) -> Mat4 {
    let eye = Vec3::new(camera.yaw.sin() * 11.0, 8.3, camera.yaw.cos() * 11.0);
    let view = Mat4::look_at_rh(eye, Vec3::new(0.0, 0.30, 0.0), Vec3::Y);
    let mut h: f32 = 1.5;
    for device in devices {
        for x in [-1.05, 1.05] {
            for y in [0.0, 1.9] {
                for z in [-0.9, 0.9] {
                    let p = view
                        .transform_point3(Vec3::from_array(device.position) + Vec3::new(x, y, z));
                    h = h.max(p.y.abs()).max(p.x.abs() / aspect.max(0.1));
                }
            }
        }
    }
    let h = h * 1.12 / camera.zoom;
    Mat4::orthographic_rh(-h * aspect, h * aspect, -h, h, 0.1, 40.0) * view
}
#[derive(Debug, Clone)]
pub struct Scene {
    pub selected: Option<String>,
    pub devices: Vec<DeviceSceneModel>,
    pub camera: Camera,
    pub dark: bool,
}
#[derive(Default)]
pub struct Interaction {
    pressed: Option<iced::Point>,
    last: Option<iced::Point>,
    dragged: bool,
}
impl shader::Program<Message> for Scene {
    type State = Interaction;
    type Primitive = ScenePrimitive;
    fn update(
        &self,
        state: &mut Interaction,
        event: &iced::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<shader::Action<Message>> {
        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(bounds) =>
            {
                state.pressed = cursor.position();
                state.last = state.pressed;
                state.dragged = false;
                Some(shader::Action::capture())
            }
            iced::Event::Mouse(mouse::Event::CursorMoved { position })
                if state.pressed.is_some() =>
            {
                if position.distance(state.pressed.unwrap()) > 5.0 {
                    state.dragged = true;
                }
                let dx = state.last.map(|p| position.x - p.x).unwrap_or(0.0);
                state.last = Some(*position);
                if state.dragged {
                    Some(shader::Action::publish(Message::Orbit(dx * 0.007)).and_capture())
                } else {
                    None
                }
            }
            iced::Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                if state.pressed.take().is_some() =>
            {
                state.last = None;
                if !state.dragged
                    && let Some(p) = cursor.position_in(bounds)
                {
                    let inv =
                        camera(self.camera, bounds.width / bounds.height, &self.devices).inverse();
                    let x = p.x / bounds.width * 2.0 - 1.0;
                    let y = 1.0 - p.y / bounds.height * 2.0;
                    let origin = inv.project_point3(Vec3::new(x, y, 0.0));
                    let direction = (inv.project_point3(Vec3::new(x, y, 1.0)) - origin).normalize();
                    let picked = self
                        .devices
                        .iter()
                        .filter_map(|d| {
                            let tall = d.kind == VisualKind::Tower;
                            let center = Vec3::from_array(d.position)
                                + Vec3::Y * if tall { 0.9 } else { 0.4 };
                            let half = if tall {
                                Vec3::new(0.6, 1.0, 0.65)
                            } else {
                                Vec3::new(0.9, 0.65, 0.85)
                            };
                            ray_box(origin, direction, center - half, center + half)
                                .map(|distance| (d, distance))
                        })
                        .min_by(|a, b| a.1.total_cmp(&b.1));
                    return picked.map(|(d, _)| {
                        shader::Action::publish(Message::Select(d.id.clone())).and_capture()
                    });
                }
                Some(shader::Action::capture())
            }
            _ => None,
        }
    }
    fn draw(&self, _: &Interaction, _: mouse::Cursor, _: Rectangle) -> ScenePrimitive {
        ScenePrimitive(self.clone())
    }
    fn mouse_interaction(
        &self,
        state: &Interaction,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if state.pressed.is_some() {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}
fn ray_box(origin: Vec3, direction: Vec3, min: Vec3, max: Vec3) -> Option<f32> {
    let a = (min - origin) / direction;
    let b = (max - origin) / direction;
    let near = a.min(b).max_element();
    let far = a.max(b).min_element();
    (far >= near.max(0.0)).then_some(near.max(0.0))
}
#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
    color: [f32; 4],
    material: [f32; 4],
}
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Pod, Zeroable)]
struct Uniform {
    camera: [[f32; 4]; 4],
    light: [[f32; 4]; 4],
    eye: [f32; 4],
    ground: [f32; 4],
    selection: [f32; 4],
    nodes: [[f32; 4]; 32],
}
#[derive(Debug)]
pub struct ScenePrimitive(Scene);
pub struct ScenePipeline {
    mesh: wgpu::Buffer,
    layout: Vec<(String, VisualKind, [f32; 3])>,
    count: u32,
    uniform: wgpu::Buffer,
    bind: wgpu::BindGroup,
    shadow_bind: wgpu::BindGroup,
    render: wgpu::RenderPipeline,
    shadow_pipeline: wgpu::RenderPipeline,
    shadow: wgpu::TextureView,
    composite: wgpu::RenderPipeline,
    composite_layout: wgpu::BindGroupLayout,
    sampler: wgpu::Sampler,
    targets: Option<Targets>,
    format: wgpu::TextureFormat,
    viewport: [f32; 4],
    last_uniform: Option<Uniform>,
    dirty: AtomicBool,
    shadow_ready: AtomicBool,
    scene_passes: AtomicU64,
    shadow_passes: AtomicU64,
    compositions: AtomicU64,
}
struct Targets {
    size: [u32; 2],
    color: wgpu::TextureView,
    msaa: wgpu::TextureView,
    depth: wgpu::TextureView,
    bind: wgpu::BindGroup,
}
impl shader::Pipeline for ScenePipeline {
    fn new(device: &wgpu::Device, _: &wgpu::Queue, format: wgpu::TextureFormat) -> Self {
        let vertices = geometry(&[]);
        let mesh = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("moonitor static device meshes"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("scene uniforms"),
            size: std::mem::size_of::<Uniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let shadow = texture(
            device,
            1024,
            1024,
            wgpu::TextureFormat::Depth32Float,
            1,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        );
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow comparison"),
            compare: Some(wgpu::CompareFunction::LessEqual),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
            ],
        });
        let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: uniform.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&shadow),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        });
        let module = device.create_shader_module(wgpu::include_wgsl!("scene.wgsl"));
        // The shadow pass cannot sample the texture it is writing.
        let shadow_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("shadow uniform only"),
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
        });
        let shadow_bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &shadow_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            }],
        });
        let shadow_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&shadow_layout],
                push_constant_ranges: &[],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&layout],
            push_constant_ranges: &[],
        });
        let buffers = [wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &wgpu::vertex_attr_array![0=>Float32x3,1=>Float32x3,2=>Float32x4,3=>Float32x4],
        }];
        let depth = wgpu::DepthStencilState {
            format: wgpu::TextureFormat::Depth32Float,
            depth_write_enabled: true,
            depth_compare: wgpu::CompareFunction::LessEqual,
            stencil: Default::default(),
            bias: Default::default(),
        };
        let render = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("lit device meshes"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_main"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            fragment: Some(wgpu::FragmentState {
                module: &module,
                entry_point: Some("fs_main"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(depth.clone()),
            multisample: wgpu::MultisampleState {
                count: 4,
                ..Default::default()
            },
            multiview: None,
            cache: None,
        });
        let shadow_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("directional shadow map"),
            layout: Some(&shadow_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &module,
                entry_point: Some("vs_shadow"),
                compilation_options: Default::default(),
                buffers: &buffers,
            },
            fragment: None,
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                bias: wgpu::DepthBiasState {
                    constant: 2,
                    slope_scale: 2.0,
                    clamp: 0.0,
                },
                ..depth
            }),
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let composite_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let composite_module = device.create_shader_module(wgpu::include_wgsl!("composite.wgsl"));
        let composite_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: None,
                bind_group_layouts: &[&composite_layout],
                push_constant_ranges: &[],
            });
        let composite = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("scene composition"),
            layout: Some(&composite_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &composite_module,
                entry_point: Some("vs"),
                compilation_options: Default::default(),
                buffers: &[],
            },
            fragment: Some(wgpu::FragmentState {
                module: &composite_module,
                entry_point: Some("fs"),
                compilation_options: Default::default(),
                targets: &[Some(format.into())],
            }),
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            multiview: None,
            cache: None,
        });
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            mesh,
            layout: Vec::new(),
            count: vertices.len() as u32,
            uniform,
            bind,
            shadow_bind,
            render,
            shadow_pipeline,
            shadow,
            composite,
            composite_layout,
            sampler,
            targets: None,
            format,
            viewport: [0.0; 4],
            last_uniform: None,
            dirty: AtomicBool::new(true),
            shadow_ready: AtomicBool::new(false),
            scene_passes: AtomicU64::new(0),
            shadow_passes: AtomicU64::new(0),
            compositions: AtomicU64::new(0),
        }
    }
}
impl shader::Primitive for ScenePrimitive {
    type Pipeline = ScenePipeline;
    fn prepare(
        &self,
        p: &mut ScenePipeline,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bounds: &Rectangle,
        viewport: &shader::Viewport,
    ) {
        let layout = self
            .0
            .devices
            .iter()
            .map(|d| (d.id.clone(), d.kind, d.position))
            .collect::<Vec<_>>();
        if p.layout != layout {
            let vertices = geometry(&self.0.devices);
            p.mesh = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("fleet meshes"),
                contents: bytemuck::cast_slice(&vertices),
                usage: wgpu::BufferUsages::VERTEX,
            });
            p.count = vertices.len() as u32;
            p.layout = layout;
            p.dirty.store(true, Ordering::Relaxed);
            p.shadow_ready.store(false, Ordering::Relaxed);
        }
        let scale = viewport.scale_factor();
        let w = (bounds.width * scale).round().max(1.0) as u32;
        let h = (bounds.height * scale).round().max(1.0) as u32;
        // Cap retina offscreen work at 1600 px wide; UI text retains full resolution.
        let ratio = (1600.0 / w as f32).min(1.0);
        let size = [(w as f32 * ratio) as u32, (h as f32 * ratio) as u32];
        if p.targets.as_ref().is_none_or(|t| t.size != size) {
            p.dirty.store(true, Ordering::Relaxed);
            let color = texture(
                device,
                size[0],
                size[1],
                p.format,
                1,
                wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            );
            let msaa = texture(
                device,
                size[0],
                size[1],
                p.format,
                4,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let depth = texture(
                device,
                size[0],
                size[1],
                wgpu::TextureFormat::Depth32Float,
                4,
                wgpu::TextureUsages::RENDER_ATTACHMENT,
            );
            let bind = device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &p.composite_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&color),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&p.sampler),
                    },
                ],
            });
            p.targets = Some(Targets {
                size,
                color,
                msaa,
                depth,
                bind,
            });
        }
        p.viewport = [
            bounds.x * scale,
            bounds.y * scale,
            bounds.width * scale,
            bounds.height * scale,
        ];
        let extent = extent(&self.0.devices);
        let light = Mat4::orthographic_rh(
            -extent * 1.5,
            extent * 1.5,
            -extent * 1.5,
            extent * 1.5,
            0.1,
            80.0,
        ) * Mat4::look_at_rh(Vec3::new(-12.0, 36.0, 20.0), Vec3::ZERO, Vec3::Y);
        let mut nodes = [[0.0; 4]; 32];
        for (i, d) in self.0.devices.iter().take(32).enumerate() {
            let color = d.status.color();
            let color = palette::resolve(
                iced::Color::from_rgb(color[0], color[1], color[2]),
                self.0.dark,
            );
            nodes[i] = [
                color.r,
                color.g,
                color.b,
                f32::from(self.0.selected.as_deref() == Some(&d.id)),
            ];
        }
        let c = self.0.camera;
        let u = Uniform {
            camera: camera(c, bounds.width / bounds.height, &self.0.devices).to_cols_array_2d(),
            light: light.to_cols_array_2d(),
            eye: [c.yaw.sin() * 11.0, 8.3, c.yaw.cos() * 11.0, 1.0],
            ground: palette::rgba(palette::resolve(palette::GROUND, self.0.dark)),
            selection: palette::rgba(palette::resolve(palette::ACCENT, self.0.dark)),
            nodes,
        };
        if p.last_uniform != Some(u) {
            queue.write_buffer(&p.uniform, 0, bytemuck::bytes_of(&u));
            p.last_uniform = Some(u);
            p.dirty.store(true, Ordering::Relaxed);
        }
    }
    fn render(
        &self,
        p: &ScenePipeline,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        clip: &Rectangle<u32>,
    ) {
        let Some(t) = &p.targets else { return };
        if !p.shadow_ready.swap(true, Ordering::Relaxed) {
            p.shadow_passes.fetch_add(1, Ordering::Relaxed);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &p.shadow,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&p.shadow_pipeline);
            pass.set_bind_group(0, &p.shadow_bind, &[]);
            pass.set_vertex_buffer(0, p.mesh.slice(..));
            pass.draw(6..p.count, 0..1);
        }
        if p.dirty.swap(false, Ordering::Relaxed) {
            let ground = palette::resolve(palette::GROUND, self.0.dark);
            p.scene_passes.fetch_add(1, Ordering::Relaxed);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("scene"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &t.msaa,
                    depth_slice: None,
                    resolve_target: Some(&t.color),
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: f64::from(ground.r).powf(2.2),
                            g: f64::from(ground.g).powf(2.2),
                            b: f64::from(ground.b).powf(2.2),
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Discard,
                    },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &t.depth,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Discard,
                    }),
                    stencil_ops: None,
                }),
                ..Default::default()
            });
            pass.set_pipeline(&p.render);
            pass.set_bind_group(0, &p.bind, &[]);
            pass.set_vertex_buffer(0, p.mesh.slice(..));
            pass.draw(0..p.count, 0..1);
        }
        p.compositions.fetch_add(1, Ordering::Relaxed);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("compose"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&p.composite);
        pass.set_bind_group(0, &t.bind, &[]);
        pass.set_viewport(
            p.viewport[0],
            p.viewport[1],
            p.viewport[2],
            p.viewport[3],
            0.0,
            1.0,
        );
        pass.set_scissor_rect(clip.x, clip.y, clip.width, clip.height);
        pass.draw(0..3, 0..1);
    }
}
impl Drop for ScenePipeline {
    fn drop(&mut self) {
        if std::env::var_os("MOONITOR_RENDER_DIAGNOSTICS").is_some() {
            eprintln!(
                "moonitor-renderer: scene_passes={} shadow_passes={} compositions={} vertices={}",
                self.scene_passes.load(Ordering::Relaxed),
                self.shadow_passes.load(Ordering::Relaxed),
                self.compositions.load(Ordering::Relaxed),
                self.count
            );
        }
    }
}
fn texture(
    d: &wgpu::Device,
    w: u32,
    h: u32,
    format: wgpu::TextureFormat,
    samples: u32,
    usage: wgpu::TextureUsages,
) -> wgpu::TextureView {
    d.create_texture(&wgpu::TextureDescriptor {
        label: Some("scene attachment"),
        size: wgpu::Extent3d {
            width: w.max(1),
            height: h.max(1),
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: samples,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage,
        view_formats: &[],
    })
    .create_view(&Default::default())
}

struct Mesh {
    vertices: Vec<Vertex>,
    node: f32,
    origin: Vec3,
    rotation: Mat4,
}
impl Mesh {
    fn vertex(&self, p: Vec3, n: Vec3, color: [f32; 3], rough: f32, kind: f32) -> Vertex {
        Vertex {
            position: (self.rotation.transform_point3(p) + self.origin).to_array(),
            normal: self.rotation.transform_vector3(n).to_array(),
            color: [color[0], color[1], color[2], 1.0],
            material: [self.node, rough, kind, 0.0],
        }
    }
    fn quad(&mut self, p: [Vec3; 4], n: Vec3, c: [f32; 3], rough: f32, kind: f32) {
        for i in [0, 1, 2, 0, 2, 3] {
            self.vertices.push(self.vertex(p[i], n, c, rough, kind));
        }
    }
    fn box3(&mut self, center: Vec3, size: Vec3, bevel: f32, c: [f32; 3], rough: f32, kind: f32) {
        let half = size * 0.5;
        let bevel = bevel.min(half.min_element());
        let inner = (half - Vec3::splat(bevel)).max(Vec3::ZERO);
        for (n, u, v) in [
            (Vec3::X, -Vec3::Z, Vec3::Y),
            (-Vec3::X, Vec3::Z, Vec3::Y),
            (Vec3::Y, Vec3::X, -Vec3::Z),
            (-Vec3::Y, Vec3::X, Vec3::Z),
            (Vec3::Z, Vec3::X, Vec3::Y),
            (-Vec3::Z, -Vec3::X, Vec3::Y),
        ] {
            let hn = n.abs().dot(half);
            let hu = u.abs().dot(half);
            let hv = v.abs().dot(half);
            if bevel <= 0.0 {
                self.quad(
                    [
                        center + n * hn - u * hu - v * hv,
                        center + n * hn + u * hu - v * hv,
                        center + n * hn + u * hu + v * hv,
                        center + n * hn - u * hu + v * hv,
                    ],
                    n,
                    c,
                    rough,
                    kind,
                );
                continue;
            }
            let coords = |h: f32| {
                [
                    -h,
                    -h + bevel * 0.3,
                    -h + bevel,
                    h - bevel,
                    h - bevel * 0.3,
                    h,
                ]
            };
            let us = coords(hu);
            let vs = coords(hv);
            for a in 0..5 {
                for b in 0..5 {
                    for (i, j) in [
                        (a, b),
                        (a + 1, b),
                        (a + 1, b + 1),
                        (a, b),
                        (a + 1, b + 1),
                        (a, b + 1),
                    ] {
                        let raw = n * hn + u * us[i] + v * vs[j];
                        let clamped = raw.clamp(-inner, inner);
                        let normal = (raw - clamped).normalize();
                        let pos = center + clamped + normal * bevel;
                        self.vertices.push(self.vertex(pos, normal, c, rough, kind));
                    }
                }
            }
        }
    }
    fn cylinder(&mut self, center: Vec3, r: f32, height: f32, c: [f32; 3], kind: f32) {
        for i in 0..80 {
            let a = i as f32 * std::f32::consts::TAU / 80.0;
            let b = (i + 1) as f32 * std::f32::consts::TAU / 80.0;
            let na = Vec3::new(a.cos(), 0.0, a.sin());
            let nb = Vec3::new(b.cos(), 0.0, b.sin());
            let pa = center + na * r;
            let pb = center + nb * r;
            self.quad(
                [pa, pb, pb + Vec3::Y * height, pa + Vec3::Y * height],
                (na + nb).normalize(),
                c,
                0.8,
                kind,
            );
            for p in [
                center + Vec3::Y * height,
                pb + Vec3::Y * height,
                pa + Vec3::Y * height,
            ] {
                self.vertices.push(self.vertex(p, Vec3::Y, c, 0.8, kind));
            }
        }
    }
}
fn geometry(devices: &[DeviceSceneModel]) -> Vec<Vertex> {
    let mut m = Mesh {
        vertices: vec![],
        node: -1.0,
        origin: Vec3::ZERO,
        rotation: Mat4::IDENTITY,
    };
    m.quad(
        [
            Vec3::new(-30.0, -0.015, 30.0),
            Vec3::new(30.0, -0.015, 30.0),
            Vec3::new(30.0, -0.015, -30.0),
            Vec3::new(-30.0, -0.015, -30.0),
        ],
        Vec3::Y,
        [0.94, 0.95, 0.93],
        1.0,
        3.0,
    );
    for (id, device) in devices.iter().enumerate() {
        m.node = id as f32;
        m.origin = Vec3::from_array(device.position);
        m.rotation = Mat4::IDENTITY;
        m.cylinder(
            Vec3::ZERO,
            if device.kind == VisualKind::Laptop {
                1.02
            } else {
                0.83
            },
            0.045,
            [0.82, 0.85, 0.81],
            2.0,
        );
        m.origin.y = 0.055;
        let silver = [0.66, 0.69, 0.76];
        let black = [0.10, 0.12, 0.17];
        let glass = [0.04, 0.05, 0.09];
        let indicator = [0.66, 0.67, 0.96];
        match device.kind {
            VisualKind::Laptop => {
                // MacBook, open lid, keyboard and trackpad.
                m.rotation = Mat4::from_rotation_y(-0.12);
                m.box3(
                    Vec3::new(0.0, 0.06, 0.0),
                    Vec3::new(1.52, 0.10, 1.02),
                    0.035,
                    silver,
                    0.25,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.115, -0.16),
                    Vec3::new(1.29, 0.012, 0.42),
                    0.004,
                    black,
                    0.8,
                    0.0,
                );
                for row in 0..4 {
                    for col in 0..13 {
                        m.box3(
                            Vec3::new(
                                -0.58 + col as f32 * 0.096,
                                0.127,
                                -0.31 + row as f32 * 0.095,
                            ),
                            Vec3::new(0.078, 0.009, 0.072),
                            0.0,
                            [0.20, 0.23, 0.24],
                            0.6,
                            0.0,
                        );
                    }
                }
                m.box3(
                    Vec3::new(0.0, 0.116, 0.29),
                    Vec3::new(0.53, 0.01, 0.26),
                    0.008,
                    [0.58, 0.64, 0.65],
                    0.4,
                    0.0,
                );
                let base = m.origin;
                m.origin += m.rotation.transform_vector3(Vec3::new(0.0, 0.13, -0.46));
                m.rotation *= Mat4::from_rotation_x(-0.20);
                m.box3(
                    Vec3::new(0.0, 0.48, 0.0),
                    Vec3::new(1.52, 0.96, 0.055),
                    0.035,
                    silver,
                    0.25,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.49, 0.031),
                    Vec3::new(1.39, 0.81, 0.008),
                    0.012,
                    glass,
                    0.1,
                    0.0,
                );
                m.box3(
                    Vec3::new(-0.27, 0.48, 0.039),
                    Vec3::new(0.63, 0.33, 0.004),
                    0.0,
                    [0.15, 0.39, 0.37],
                    0.4,
                    1.0,
                );
                m.box3(
                    Vec3::new(0.33, 0.48, 0.040),
                    Vec3::new(0.44, 0.33, 0.004),
                    0.0,
                    [0.25, 0.54, 0.46],
                    0.4,
                    1.0,
                );
                m.origin = base;
            }
            VisualKind::Tower => {
                // Exon workstation, perforated front and side panel.
                m.rotation = Mat4::from_rotation_y(0.12);
                m.box3(
                    Vec3::new(0.0, 0.93, 0.0),
                    Vec3::new(0.79, 1.82, 0.98),
                    0.055,
                    [0.12, 0.15, 0.16],
                    0.55,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.94, 0.502),
                    Vec3::new(0.65, 1.62, 0.025),
                    0.018,
                    [0.07, 0.10, 0.11],
                    0.9,
                    0.0,
                );
                for row in 0..20 {
                    for col in 0..6 {
                        m.box3(
                            Vec3::new(
                                -0.245 + col as f32 * 0.098,
                                0.26 + row as f32 * 0.066,
                                0.522,
                            ),
                            Vec3::new(0.052, 0.018, 0.005),
                            0.0,
                            [0.21, 0.26, 0.27],
                            0.9,
                            0.0,
                        );
                    }
                }
                m.box3(
                    Vec3::new(0.405, 0.96, 0.0),
                    Vec3::new(0.013, 1.49, 0.78),
                    0.003,
                    [0.16, 0.21, 0.22],
                    0.2,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 1.74, 0.52),
                    Vec3::new(0.11, 0.018, 0.006),
                    0.0,
                    indicator,
                    0.2,
                    1.0,
                );
            }
            VisualKind::Compact => {
                // DGX Spark: compact champagne metal chassis with fins.
                m.rotation = Mat4::from_rotation_y(-0.08);
                m.box3(
                    Vec3::new(0.0, 0.25, 0.0),
                    Vec3::new(1.14, 0.42, 1.09),
                    0.055,
                    [0.63, 0.58, 0.46],
                    0.35,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.472, 0.0),
                    Vec3::new(0.95, 0.014, 0.89),
                    0.007,
                    [0.76, 0.71, 0.59],
                    0.35,
                    0.0,
                );
                for i in 0..17 {
                    m.box3(
                        Vec3::new(-0.46 + i as f32 * 0.057, 0.29, 0.554),
                        Vec3::new(0.018, 0.19, 0.01),
                        0.0,
                        [0.19, 0.19, 0.15],
                        0.9,
                        0.0,
                    );
                }
                m.box3(
                    Vec3::new(0.34, 0.13, 0.56),
                    Vec3::new(0.045, 0.012, 0.006),
                    0.0,
                    indicator,
                    0.2,
                    1.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.484, 0.0),
                    Vec3::new(0.21, 0.008, 0.21),
                    0.008,
                    [0.22, 0.25, 0.17],
                    0.5,
                    0.0,
                );
            }
            VisualKind::Board => {
                // Zima board with heatsink, ports and visible circuit board.
                m.rotation = Mat4::from_rotation_y(-0.20);
                m.box3(
                    Vec3::new(0.0, 0.105, 0.0),
                    Vec3::new(1.18, 0.08, 0.79),
                    0.025,
                    [0.14, 0.18, 0.28],
                    0.7,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.20, -0.04),
                    Vec3::new(0.80, 0.17, 0.56),
                    0.015,
                    [0.25, 0.31, 0.30],
                    0.45,
                    0.0,
                );
                for i in 0..13 {
                    m.box3(
                        Vec3::new(-0.38 + i as f32 * 0.064, 0.33, -0.04),
                        Vec3::new(0.027, 0.15, 0.56),
                        0.0,
                        [0.46, 0.53, 0.51],
                        0.45,
                        0.0,
                    );
                }
                for i in 0..3 {
                    m.box3(
                        Vec3::new(-0.36 + i as f32 * 0.32, 0.21, 0.35),
                        Vec3::new(0.24, 0.20, 0.16),
                        0.012,
                        silver,
                        0.4,
                        0.0,
                    );
                    m.box3(
                        Vec3::new(-0.36 + i as f32 * 0.32, 0.21, 0.438),
                        Vec3::new(0.17, 0.11, 0.012),
                        0.0,
                        black,
                        0.8,
                        0.0,
                    );
                }
            }
            VisualKind::Phone => {
                // iPhone glass slab, elevated enough for readable silhouette.
                m.rotation = Mat4::from_rotation_y(-0.25) * Mat4::from_rotation_x(-0.12);
                m.box3(
                    Vec3::new(0.0, 0.14, 0.0),
                    Vec3::new(0.63, 0.095, 1.22),
                    0.045,
                    [0.21, 0.25, 0.27],
                    0.25,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.191, 0.0),
                    Vec3::new(0.57, 0.012, 1.14),
                    0.04,
                    glass,
                    0.08,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.200, 0.04),
                    Vec3::new(0.48, 0.005, 0.80),
                    0.035,
                    [0.17, 0.24, 0.38],
                    0.5,
                    0.0,
                );
                m.box3(
                    Vec3::new(0.0, 0.201, -0.47),
                    Vec3::new(0.22, 0.006, 0.058),
                    0.022,
                    black,
                    0.4,
                    0.0,
                );
            }
        }
    }
    m.vertices
}

fn extent(devices: &[DeviceSceneModel]) -> f32 {
    devices
        .iter()
        .map(|d| d.position[0].abs().max(d.position[2].abs()) + 2.0)
        .fold(2.4, f32::max)
        * 1.25
}

impl iced::widget::canvas::Program<Message> for Scene {
    type State = ();
    fn draw(
        &self,
        _: &(),
        renderer: &iced::Renderer,
        _: &iced::Theme,
        bounds: Rectangle,
        _: mouse::Cursor,
    ) -> Vec<iced::widget::canvas::Geometry> {
        use iced::widget::canvas;
        let mut frame = canvas::Frame::new(renderer, bounds.size());
        let mut labels: Vec<Rectangle> = Vec::new();
        let matrix = camera(self.camera, bounds.width / bounds.height, &self.devices);
        for (i, d) in self.devices.iter().enumerate() {
            let p = matrix.project_point3(
                Vec3::from_array(d.position)
                    + Vec3::new(
                        0.0,
                        if d.kind == VisualKind::Tower {
                            2.0
                        } else {
                            1.15
                        },
                        0.0,
                    ),
            );
            let label = if self.devices.len() > 8 || bounds.width < 300.0 {
                format!("{:02}", i + 1)
            } else {
                format!("{:02} {}", i + 1, d.label)
            };
            let width = label.chars().count() as f32 * 5.7 + 14.0;
            let x = ((p.x + 1.0) * bounds.width / 2.0)
                .clamp(width / 2.0, (bounds.width - width / 2.0).max(width / 2.0));
            let mut y = ((1.0 - p.y) * bounds.height / 2.0 - 12.0)
                .clamp(10.0, (bounds.height - 10.0).max(10.0));
            // Separate labels that project close together; never cover another name.
            let desired_y = y;
            for step in 0..20 {
                let shift = if step % 2 == 0 {
                    step as f32 * 10.0
                } else {
                    -(step as f32 + 1.0) * 10.0
                };
                let candidate = (desired_y + shift).clamp(10.0, (bounds.height - 10.0).max(10.0));
                let rect = Rectangle {
                    x: x - width / 2.0 - 2.0,
                    y: candidate - 10.0,
                    width: width + 4.0,
                    height: 20.0,
                };
                if labels.iter().all(|other| !rect.intersects(other)) {
                    y = candidate;
                    break;
                }
            }
            labels.push(Rectangle {
                x: x - width / 2.0 - 2.0,
                y: y - 10.0,
                width: width + 4.0,
                height: 20.0,
            });
            let plate = canvas::Path::rounded_rectangle(
                iced::Point::new(x - width / 2.0, y - 9.0),
                iced::Size::new(width, 18.0),
                4.0.into(),
            );
            frame.fill(&plate, palette::resolve(palette::PAPER, self.dark));
            let c = d.status.color();
            frame.stroke(
                &plate,
                canvas::Stroke::default()
                    .with_width(0.8)
                    .with_color(palette::resolve(
                        iced::Color::from_rgb(c[0], c[1], c[2]),
                        self.dark,
                    )),
            );
            frame.fill_text(canvas::Text {
                content: label,
                position: iced::Point::new(x, y),
                font: iced::Font::with_name("Inter"),
                align_y: iced::alignment::Vertical::Center,
                color: palette::resolve(palette::INK, self.dark),
                size: 11.0.into(),
                align_x: iced::alignment::Horizontal::Center.into(),
                ..Default::default()
            });
        }
        vec![frame.into_geometry()]
    }
}
#[cfg(test)]
impl Scene {
    pub fn test_device_point(&self, index: usize, size: iced::Size) -> iced::Point {
        let d = &self.devices[index];
        let p = camera(self.camera, size.width / size.height, &self.devices)
            .project_point3(Vec3::from_array(d.position) + Vec3::Y * 0.5);
        iced::Point::new(
            (p.x + 1.0) * size.width / 2.0,
            (1.0 - p.y) * size.height / 2.0,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view_model::NodeStatus;
    #[test]
    fn mesh_contains_exact_fleet_ids_and_finite_geometry() {
        for count in [0, 1, 2, 5, 16, 32] {
            let devices = (0..count)
                .map(|i| DeviceSceneModel {
                    id: format!("test-{i}"),
                    label: format!("Test {i}"),
                    kind: [
                        VisualKind::Laptop,
                        VisualKind::Tower,
                        VisualKind::Compact,
                        VisualKind::Board,
                        VisualKind::Phone,
                    ][i % 5],
                    status: NodeStatus::Uninstrumented,
                    position: [i as f32 * 2.6, 0.0, 0.0],
                })
                .collect::<Vec<_>>();
            let vertices = geometry(&devices);
            let ids = vertices
                .iter()
                .filter(|v| v.material[0] >= 0.0)
                .map(|v| v.material[0] as usize)
                .collect::<std::collections::BTreeSet<_>>();
            assert_eq!(ids, (0..count).collect());
            assert!(vertices.iter().all(|v| {
                v.position
                    .iter()
                    .chain(v.normal.iter())
                    .all(|n| n.is_finite())
            }));
        }
    }
}
