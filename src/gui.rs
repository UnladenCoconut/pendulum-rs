use std::{fmt::format, ops::RangeInclusive};

use egui_wgpu::ScreenDescriptor;
use strum::IntoEnumIterator;
use wgpu::{
    CommandEncoder, Device, PrimitiveTopology, Queue, RenderPassColorAttachment, RenderPassDescriptor, TextureFormat, rwh::HasDisplayHandle,
};

use crate::geometry::Geometries;

#[cfg(not(target_arch = "wasm32"))]
use std::time::{Instant, Duration};

#[cfg(target_arch = "wasm32")]
use web_time::{Instant, Duration};

pub struct GUI {
    pub context: egui::Context,
    pub state: egui_winit::State,
    pub renderer: egui_wgpu::Renderer,

    //data
    pub mesh_dirty: bool,
    pub subdivisions: u32,
    pub wireframe_mode: bool,
    pub wireframe_mode_dirty: bool,

    pub selected_geometry: Geometries,
    pub selected_geometry_dirty: bool,

    pub frame_time_max: Duration,

}

impl GUI {
    
    pub fn new(device: &Device, display_target: &dyn HasDisplayHandle, format: TextureFormat) -> Self {
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
                format,
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
            selected_geometry: Geometries::Sphere,
            selected_geometry_dirty: true,

            frame_time_max: Duration::ZERO,

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
        dt: Duration,
    ) {
        let input = self.state.take_egui_input(window);
        let mut output = self.context.run_ui(input, |_x| {
            egui::Window::new("Controls")
                .default_size(egui::vec2(100.0, 800.0))
                .default_pos(egui::pos2(700.0, 100.0))
                .default_open(true)
                .show(&self.context, |ui| {

                    if dt > self.frame_time_max {
                        self.frame_time_max = dt;
                    }

                    let dt_ms = dt.as_millis();
                    let fps = if dt_ms != 0 {
                        1000u128/dt_ms
                    } else {
                        0
                    };
                    ui.label(format!("Framerate: {} FPS",fps));
                    ui.label(format!("Frame time: {}ms",dt.as_millis()));
                    ui.label(format!("Frame time max: {}ms",self.frame_time_max.as_millis()));

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


                    let before = self.selected_geometry.clone();
                    egui::ComboBox::from_label("Mesh Geometry")
                    .selected_text(self.selected_geometry.to_string())
                    .show_ui(ui, |ui| {
                        Geometries::iter().for_each(|e| {
                            ui.selectable_value(
                                &mut self.selected_geometry,
                                e.clone(),
                                e.to_string(),
                            );
                        });
                    });
                    if before != self.selected_geometry {
                        self.mesh_dirty = true;
                    }

                });
        });

        let primitives = self
            .context
            .tessellate(output.shapes, output.pixels_per_point);
        output.textures_delta.set.iter().for_each(|(id, delta)| {
            self.renderer.update_texture(device, queue, *id, delta);
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
