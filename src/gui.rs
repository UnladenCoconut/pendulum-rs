use std::ops::RangeInclusive;

use egui_wgpu::ScreenDescriptor;
use wgpu::{
    CommandEncoder, Device, Queue, RenderPassColorAttachment, RenderPassDescriptor,
    rwh::HasDisplayHandle,
};

pub struct GUI {
    pub context: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,

    //data
    pub mesh_dirty: bool,
    pub subdivisions: u32,
    pub wireframe_mode: bool,
    pub wireframe_mode_dirty: bool,
}

impl GUI {
    pub fn polygon_mode(&self) -> wgpu::PolygonMode {
        match self.wireframe_mode {
            true => wgpu::PolygonMode::Line,
            false => wgpu::PolygonMode::Fill,
        }
    }

    pub fn new(device: &Device, display_target: &dyn HasDisplayHandle) -> Self {
        let ctx = egui::Context::default();
        Self {
            state: egui_winit::State::new(
                ctx.clone(),
                egui::ViewportId::ROOT,
                display_target,
                None,
                None,
                None,
            ),
            context: ctx,
            renderer: egui_wgpu::Renderer::new(
                device,
                wgpu::TextureFormat::Rgba8Unorm,
                egui_wgpu::RendererOptions {
                    msaa_samples: 1,
                    depth_stencil_format: None,
                    dithering: false,
                    predictable_texture_filtering: false,
                },
            ),

            subdivisions: 2,
            mesh_dirty: true,
            wireframe_mode: true,
            wireframe_mode_dirty: false,
        }
    }

    pub fn render(
        &mut self,
        device: &Device,
        queue: &Queue,
        encoder: &mut CommandEncoder,
        view: &wgpu::TextureView,
        window: &winit::window::Window,
        screen_desc: ScreenDescriptor,
    ) {
        let input = self.state.take_egui_input(window);
        let mut output = self.context.run_ui(input, |_x| {
            egui::Window::new("Controls")
                .default_size(egui::vec2(100.0, 800.0))
                .default_pos(egui::pos2(700.0, 100.0))
                .default_open(true)
                .show(&self.context, |ui| {
                    if ui
                        .checkbox(&mut self.wireframe_mode, "wireframe mode")
                        .changed()
                    {
                        self.wireframe_mode_dirty |= true;
                        ui.ctx().request_repaint();
                    }
                    ui.add(egui::Label::new("mesh subdivisions:"));
                    let response = ui.add(egui::Slider::new(
                        &mut self.subdivisions,
                        RangeInclusive::new(0, 10),
                    ));
                    if response.changed() {
                        self.mesh_dirty |= true;
                        ui.ctx().request_repaint();
                    }
                });
        });

        let primitives = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        output.textures_delta.set.iter().for_each(|(id, deltas)| {
            deltas.iter().for_each(|delta| {
                self.renderer.update_texture(device, queue, *id, delta);
            });
        });

        output.textures_delta.free.iter().for_each(|id| {
            self.renderer.free_texture(id);
        });

        output.textures_delta.clear();

        self.renderer
            .update_buffers(device, queue, encoder, &primitives, &screen_desc);

        {
            let render_pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("EGUI Render Pass"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.renderer.render(
                &mut render_pass.forget_lifetime(),
                &primitives,
                &screen_desc,
            );
        }
    }
}
