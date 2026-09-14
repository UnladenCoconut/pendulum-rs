use std::{num::NonZero, sync::Arc};

use bytemuck::cast_slice;
use glam::Mat4;
use wgpu::{
    util::{BufferInitDescriptor, DeviceExt, new_instance_with_webgpu_detection},
    *,
};
use winit::window::Window;

use crate::geometry::*;
use crate::gui::GUI;

pub struct Renderer {
    pub _instance: Instance,
    pub device: Device,
    pub queue: Queue,
    pub surface: Surface<'static>,
    pub surface_config: SurfaceConfiguration,
    pub shader: ShaderModule,
    pub render_pipeline: RenderPipeline,
    pub mesh_alloc: MeshAllocation,
    pub bind_layout: BindGroupLayout,
    pub format: TextureFormat,

    pub transform_bg: BindGroup,
    pub camera_transform_buffer: Buffer,
    pub object_transform_buffer: Buffer,

    pub gui: GUI,

    //make sure this is the last field in the struct so that it is dropped last, s.t.
    // wgpu context is dropeed before closing winit context
    //otherwise will segfault on app close
    //TODO this shouldnt be here anyway if you got your lifetimes right
    pub window: Arc<Window>,
}

impl Renderer {
    pub fn vtx_buffer(device: &Device, vtx_data: &Vec<Vertex>) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(vtx_data),
            usage: BufferUsages::VERTEX,
        })
    }

    pub fn mesh_alloc(device: &Device, geometry: Geometries, subdivisions: u32) -> MeshAllocation {
        MeshAllocation::new(device, &geometry.mesh(subdivisions))
    }

    pub fn render_pipeline(
        shader: &ShaderModule,
        device: &Device,
        bind_layouts: &[Option<&BindGroupLayout>],
        format: TextureFormat,
        primitive_state: PrimitiveState,
    ) -> RenderPipeline {
        let render_pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: Some("Render Pipeline Layout"),
            bind_group_layouts: bind_layouts,
            immediate_size: 0,
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[MeshDataDescriptor::LAYOUT], //TODO add vertex buffers here
                compilation_options: PipelineCompilationOptions::default(),
            },
            primitive: primitive_state,
            depth_stencil: None,
            cache: None,
            multiview_mask: None,
            multisample: MultisampleState::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                compilation_options: PipelineCompilationOptions::default(),
                targets: &[Some(ColorTargetState {
                    format: format,
                    blend: None,
                    write_mask: ColorWrites::ALL,
                })],
            }),
        })
    }

    pub async fn new(window: Window) -> Renderer {
        let window = Arc::new(window);

        let instance = new_instance_with_webgpu_detection(
            InstanceDescriptor::new_without_display_handle(),
        ).await;

        {
            let surface = instance.create_surface(window.clone()).unwrap();
            let rq_opts = RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            };
            let adapter = instance.request_adapter(&rq_opts).await.unwrap();

            let device_desc = DeviceDescriptor {
                label: Some("Device Descriptor"),
                required_features: Features::empty(), //TODO Features::POLYGON_MODE_LINE not supported on webgpu
                required_limits: wgpu::Limits::defaults(),
                experimental_features: ExperimentalFeatures::disabled(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            };

            let (device, queue) = adapter.request_device(&device_desc).await.unwrap();

            let size = window.inner_size();

            let caps = surface.get_capabilities(&adapter);
            let format = TextureFormat::Rgba8Unorm;

            if !caps.formats.contains(&format) {
                panic!("format {:?} not in list of supported formats", format);
            }

            let surface_config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: format,
                width: 800,
                height: 800,
                present_mode: wgpu::PresentMode::AutoNoVsync,
                desired_maximum_frame_latency: 2,
                alpha_mode: wgpu::CompositeAlphaMode::Opaque,
                view_formats: vec![format],
            };

            surface.configure(&device, &surface_config);

            ////////////////////// shader stuff //////////////////////

            let shader = device
                .create_shader_module(include_spirv!(concat!(env!("OUT_DIR"), "/shader.spirv")));

            let camera_transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Camera Matrix Uniform"),
                contents: cast_slice(Mat4::IDENTITY.as_ref()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let object_transform_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("Object World Transform Matrix Uniform"),
                contents: cast_slice(Mat4::IDENTITY.as_ref()),
                usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            });

            let bind_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
                label: Some("Camera Bind Group Layout"),
                entries: &[
                    BindGroupLayoutEntry {
                        binding: 0,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZero::new(size_of::<glam::Mat4>() as u64),
                        },
                        count: None,
                    },
                    BindGroupLayoutEntry {
                        binding: 1,
                        visibility: ShaderStages::VERTEX,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: NonZero::new(size_of::<glam::Mat4>() as u64),
                        },
                        count: None,
                    },
                ],
            });

            let bind_group = device.create_bind_group(&BindGroupDescriptor {
                label: Some("camera Bind Group"),
                layout: &bind_layout,
                entries: &[
                    BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(
                            camera_transform_buffer.as_entire_buffer_binding(),
                        ),
                    },
                    BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Buffer(
                            object_transform_buffer.as_entire_buffer_binding(),
                        ),
                    },
                ],
            });

            ////////////////////////////////////////////////////////
            let gui = GUI::new(&device, &window);

            let render_pipeline = Self::render_pipeline(
                &shader,
                &device,
                &[Some(&bind_layout)],
                format,
                PrimitiveState {
                    polygon_mode: gui.polygon_mode(),
                    ..MeshDataDescriptor::PRIMITIVE_STATE
                },
            );

            Renderer {
                window: window,
                _instance: instance,
                mesh_alloc: Self::mesh_alloc(&device, gui.selected_geometry, gui.subdivisions),
                device: device,
                queue: queue,
                surface: surface,
                surface_config: surface_config,
                shader: shader,
                render_pipeline: render_pipeline,

                transform_bg: bind_group,
                camera_transform_buffer: camera_transform_buffer,
                object_transform_buffer: object_transform_buffer,

                bind_layout: bind_layout,
                format: format,

                gui: gui,
            }
        }
    }

    pub fn pass(&mut self) {
        if self.gui.wireframe_mode_dirty {
            self.render_pipeline = Self::render_pipeline(
                &self.shader,
                &self.device,
                &[Some(&self.bind_layout)],
                self.format,
                PrimitiveState {
                    polygon_mode: self.gui.polygon_mode(),
                    ..MeshDataDescriptor::PRIMITIVE_STATE
                },
            );
        }

        let output = self.surface.get_current_texture();
        if let CurrentSurfaceTexture::Success(output) = output {
            let view = output
                .texture
                .create_view(&TextureViewDescriptor::default());
            let mut encoder = self
                .device
                .create_command_encoder(&CommandEncoderDescriptor::default());

            {
                let mut render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                    label: Some("Render Pass"),
                    color_attachments: &[Some(RenderPassColorAttachment {
                        view: &view,
                        depth_slice: None,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color {
                                r: 0.0,
                                g: 0.0,
                                b: 0.0,
                                a: 1.0,
                            }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    multiview_mask: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
                render_pass.set_pipeline(&self.render_pipeline);
                render_pass.set_bind_group(0, &self.transform_bg, &[]);

                //update buffers
                {}

                if self.gui.mesh_dirty {
                    self.mesh_alloc = Self::mesh_alloc(&self.device, self.gui.selected_geometry, self.gui.subdivisions);
                    render_pass.set_vertex_buffer(
                        MeshDataDescriptor::ATTRIBUTE.shader_location,
                        self.mesh_alloc.vtx_buffer.slice(..),
                    );

                    render_pass.set_index_buffer(
                        self.mesh_alloc.idx_buffer.slice(..),
                        MeshDataDescriptor::INDEX_FORMAT,
                    );
                }

                //render_pass.draw(0..self.mesh_alloc.vtx_count, 0..1);
                render_pass.draw_indexed(0..self.mesh_alloc.idx_count, 0, 0..1);
            }

            self.gui.render(
                &self.device,
                &self.queue,
                &mut encoder,
                &view,
                &self.window,
                egui_wgpu::ScreenDescriptor {
                    size_in_pixels: [self.surface_config.width, self.surface_config.height],
                    pixels_per_point: self.window.scale_factor() as f32,
                },
            );

            self.queue.submit(std::iter::once(encoder.finish()));
            output.present();
        } else {
            log::warn!("not handled: {:?}", output);
        }
        self.window.request_redraw();
    }
}
