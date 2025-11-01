use wgpu::util::DeviceExt;
use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::Window,
};
use std::sync::Arc;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Uniforms {
    time: f32,
    shader_type: u32,
    resolution: [f32; 2],
    planet_position: [f32; 2],
    planet_scale: f32,
    _padding: f32,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
struct Vertex {
    position: [f32; 3],
    normal: [f32; 3],
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

fn create_sphere(subdivisions: u32) -> (Vec<Vertex>, Vec<u16>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for lat in 0..=subdivisions {
        let theta = lat as f32 * std::f32::consts::PI / subdivisions as f32;
        let sin_theta = theta.sin();
        let cos_theta = theta.cos();

        for lon in 0..=subdivisions {
            let phi = lon as f32 * 2.0 * std::f32::consts::PI / subdivisions as f32;
            let sin_phi = phi.sin();
            let cos_phi = phi.cos();

            let x = sin_theta * cos_phi;
            let y = cos_theta;
            let z = sin_theta * sin_phi;

            vertices.push(Vertex {
                position: [x, y, z],
                normal: [x, y, z],
            });
        }
    }

    for lat in 0..subdivisions {
        for lon in 0..subdivisions {
            let first = (lat * (subdivisions + 1) + lon) as u16;
            let second = first + subdivisions as u16 + 1;

            indices.push(first);
            indices.push(second);
            indices.push(first + 1);

            indices.push(second);
            indices.push(second + 1);
            indices.push(first + 1);
        }
    }

    (vertices, indices)
}

struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    vertex_buffer: wgpu::Buffer,
    index_buffer: wgpu::Buffer,
    num_indices: u32,
    uniform_buffer: wgpu::Buffer,
    uniform_bind_group: wgpu::BindGroup,
    uniforms: Uniforms,
    camera_rotation: [f32; 2], // Add camera rotation angles
    start_time: std::time::Instant,
}

impl State {
    async fn new(window: Arc<Window>) -> Self {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance.create_surface(window.clone()).unwrap();

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
                    required_limits: wgpu::Limits::default(),
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
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
            present_mode: surface_caps
                .present_modes
                .iter()
                .copied()
                .find(|m| m == &wgpu::PresentMode::Fifo)
                .unwrap_or(surface_caps.present_modes[0]),
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let (vertices, indices) = create_sphere(50);
        let num_indices = indices.len() as u32;

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let uniforms = Uniforms {
            time: 0.0,
            shader_type: 1,
            resolution: [size.width as f32, size.height as f32],
            planet_position: [0.0, 0.0],
            planet_scale: 0.3,
            _padding: 0.0,
        };

        let uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Uniform Buffer"),
            contents: bytemuck::cast_slice(&[uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let uniform_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform_buffer.as_entire_binding(),
            }],
            label: Some("uniform_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&uniform_bind_group_layout],
                push_constant_ranges: &[],
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
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: Some(wgpu::Face::Back),
                polygon_mode: wgpu::PolygonMode::Fill,
                unclipped_depth: false,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        });

        Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            vertex_buffer,
            index_buffer,
            num_indices,
            uniform_buffer,
            uniform_bind_group,
            uniforms,
            camera_rotation: [0.0, 0.0],
            start_time: std::time::Instant::now(),
        }
    }

    fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.uniforms.resolution = [new_size.width as f32, new_size.height as f32];
        }
    }

    fn input(&mut self, event: &KeyEvent) -> bool {
        match event.physical_key {
            PhysicalKey::Code(KeyCode::ArrowLeft) => {
                self.camera_rotation[0] -= 0.1;
                true
            }
            PhysicalKey::Code(KeyCode::ArrowRight) => {
                self.camera_rotation[0] += 0.1;
                true
            }
            PhysicalKey::Code(KeyCode::ArrowUp) => {
                self.camera_rotation[1] = (self.camera_rotation[1] + 0.1).min(1.5);
                true
            }
            PhysicalKey::Code(KeyCode::ArrowDown) => {
                self.camera_rotation[1] = (self.camera_rotation[1] - 0.1).max(-1.5);
                true
            }
            _ => false,
        }
    }

    fn update(&mut self) {
        self.uniforms.time = self.start_time.elapsed().as_secs_f32();
        self.queue.write_buffer(
            &self.uniform_buffer,
            0,
            bytemuck::cast_slice(&[self.uniforms]),
        );
    }

    fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        // Define planets data: [x, y, scale, shader_type]
        let planets = [
            [0.0, 0.0, 0.6, 2.0],    // Sol -> Luna (centro, grande, hielo)
            [-0.4, 0.3, 0.13, 5.0],  // Rocoso -> Volcánico (izq arriba)
            [-0.6, -0.4, 0.18, 4.0], // Volcánico -> Anillos (izq abajo)
            [0.6, 0.2, 0.3, 1.0],    // Gaseoso -> Sol (der arriba)
            [0.4, -0.1, 0.05, 6.0],  // Anillos -> Rocoso (der abajo)
            [0.3, -0.6, 0.12, 3.0],  // Luna -> Gaseoso (abajo centro)
        ];

        // Create planet buffers and bind groups
        let planet_data: Vec<_> = planets
            .iter()
            .map(|planet| {
                let mut uniforms = self.uniforms;
                uniforms.planet_position = [planet[0], planet[1]];
                uniforms.planet_scale = planet[2];
                uniforms.shader_type = planet[3] as u32;

                let uniform_buffer = self.device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                    label: Some("Planet Uniform Buffer"),
                    contents: bytemuck::cast_slice(&[uniforms]),
                    usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                });

                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    layout: &self.render_pipeline.get_bind_group_layout(0),
                    entries: &[wgpu::BindGroupEntry {
                        binding: 0,
                        resource: uniform_buffer.as_entire_binding(),
                    }],
                    label: Some("Planet Bind Group"),
                });

                (uniform_buffer, bind_group)
            })
            .collect();

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.01,
                            g: 0.01,
                            b: 0.03,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                occlusion_query_set: None,
                timestamp_writes: None,
            });

            // Set the pipeline and vertex/index buffers
            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            render_pass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint16);

            // Draw stars
            for i in 0..200 {
                let x = (i as f32 * 567.123).sin() * 2.0;
                let y = (i as f32 * 432.567).cos() * 2.0;
                let size = ((i as f32 * 789.345).sin() * 0.5 + 0.5) * 0.003;
                
                let mut star_uniforms = self.uniforms;
                star_uniforms.planet_position = [x, y];
                star_uniforms.planet_scale = size;
                star_uniforms.shader_type = 7;

                self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::cast_slice(&[star_uniforms]));
                render_pass.set_bind_group(0, &self.uniform_bind_group, &[]);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }

            // Draw planets with camera rotation
            for (i, (buffer, bind_group)) in planet_data.iter().enumerate() {
                let planet = planets[i];

                // Apply camera rotation to planet position
                let mut planet_uniforms = self.uniforms;
                planet_uniforms.planet_position = [
                    planet[0] * self.camera_rotation[0].cos() - planet[2] * self.camera_rotation[0].sin(),
                    planet[1] * self.camera_rotation[1].cos()
                ];
                planet_uniforms.planet_scale = planet[2] * 
                    (0.8 + 0.2 * (self.camera_rotation[0].cos() * self.camera_rotation[1].cos()));
                planet_uniforms.shader_type = planet[3] as u32;

                self.queue.write_buffer(buffer, 0, bytemuck::cast_slice(&[planet_uniforms]));
                render_pass.set_bind_group(0, bind_group, &[]);
                render_pass.draw_indexed(0..self.num_indices, 0, 0..1);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        Ok(())
    }
}

fn main() {
    env_logger::init();
    
    let event_loop = EventLoop::new().unwrap();
    let window = Arc::new(
        winit::window::WindowBuilder::new()
            .with_title("Sistema Solar d Batman")
            .with_inner_size(winit::dpi::LogicalSize::new(1000, 800))
            .build(&event_loop)
            .unwrap(),
    );

    let mut state = pollster::block_on(State::new(window.clone()));

    println!("ESC: Salir");

    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == window.id() => match event {
                    WindowEvent::CloseRequested
                    | WindowEvent::KeyboardInput {
                        event:
                            KeyEvent {
                                state: ElementState::Pressed,
                                physical_key: PhysicalKey::Code(KeyCode::Escape),
                                ..
                            },
                        ..
                    } => control_flow.exit(),
                    WindowEvent::Resized(physical_size) => {
                        state.resize(*physical_size);
                    }
                    WindowEvent::KeyboardInput { event, .. } => {
                        state.input(event);
                    }
                    WindowEvent::RedrawRequested => {
                        state.update();
                        match state.render() {
                            Ok(_) => {}
                            Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                            Err(wgpu::SurfaceError::OutOfMemory) => control_flow.exit(),
                            Err(e) => eprintln!("{:?}", e),
                        }
                    }
                    _ => {}
                },
                Event::AboutToWait => {
                    window.request_redraw();
                }
                _ => {}
            }
        })
        .unwrap();
}
