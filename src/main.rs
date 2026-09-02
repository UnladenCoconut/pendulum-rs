#![allow(dead_code)]

use bit_set::BitSet;
use bytemuck::bytes_of;
use std::error::Error;
use std::f32::consts::PI;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode::{self};
use winit::keyboard::PhysicalKey::{self};
use winit::window::{Window, WindowId};

mod camera;
mod geometry;
mod gui;
mod move_controller;
mod renderer;
mod util;

use camera::*;
use move_controller::*;
use renderer::*;

//which item is currently controllable with wasd/arrow keys.
#[derive(Debug)]
enum ActiveMoveItem {
    Camera,
    Cube,
}

struct App {
    renderer: Option<Renderer>,
    pressed_keys: BitSet,
    cube_move_control: MoveControl,
    active_move_item: ActiveMoveItem,
    camera: Camera,
}

impl Default for App {
    fn default() -> Self {
        let keybinds = MoveControlKeySet {
            rot_x_ccw: KeyCode::KeyW,
            rot_x_cw: KeyCode::KeyS,
            rot_y_ccw: KeyCode::KeyA,
            rot_y_cw: KeyCode::KeyD,
            rot_z_ccw: KeyCode::KeyQ,
            rot_z_cw: KeyCode::KeyE,

            trans_x_dec: KeyCode::ArrowLeft,
            trans_x_inc: KeyCode::ArrowRight,
            trans_y_dec: KeyCode::ArrowDown,
            trans_y_inc: KeyCode::ArrowUp,
            trans_z_dec: KeyCode::Minus,
            trans_z_inc: KeyCode::Equal,
        };

        let shared_move_control = MoveControl::new(keybinds, 30.0, PI);

        Self {
            renderer: None,
            pressed_keys: BitSet::default(),
            camera: Camera::new(shared_move_control.clone()),
            cube_move_control: shared_move_control,
            active_move_item: ActiveMoveItem::Cube,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if let None = self.renderer {
            let window = event_loop
                .create_window(Window::default_attributes().with_inner_size(Size::Logical(
                    LogicalSize {
                        width: 800.0,
                        height: 800.0,
                    },
                )))
                .unwrap();

            let renderer = Renderer::new(window);

            //TODO this is becoming mesy, we do this in two places
            renderer.queue.write_buffer(
                &renderer.camera_transform_buffer,
                0,
                bytes_of(&self.camera.update(&self.pressed_keys)),
            );

            self.renderer.replace(renderer);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event.clone() {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                if self.pressed_keys.contains(KeyCode::Backquote as usize) {
                    log::info!("Active item: camera");
                    self.active_move_item = ActiveMoveItem::Camera;
                }
                if self.pressed_keys.contains(KeyCode::Digit1 as usize) {
                    log::info!("Active item: cube");
                    self.active_move_item = ActiveMoveItem::Cube;
                }

                if let Some(renderer) = &mut self.renderer {
                    //TODO we some better way to do this, e.g. a common trait fn, or behaviour in MoveControl so we can add Z mapping and projetion

                    match self.active_move_item {
                        ActiveMoveItem::Camera => {
                            if self.pressed_keys.contains(KeyCode::Enter as usize) {
                                self.camera.reset();
                            }
                            renderer.queue.write_buffer(
                                &renderer.camera_transform_buffer,
                                0,
                                bytes_of(&self.camera.update(&self.pressed_keys)),
                            );
                            renderer.pass();
                        }
                        ActiveMoveItem::Cube => {
                            if self.pressed_keys.contains(KeyCode::Enter as usize) {
                                self.cube_move_control.reset();
                            }
                            renderer.queue.write_buffer(
                                &renderer.object_transform_buffer,
                                0,
                                bytes_of(
                                    &self.cube_move_control.update(&self.pressed_keys).inverse(),
                                ),
                            );
                            renderer.pass();
                        }
                    };
                }
            }

            WindowEvent::Resized(size) => {
                if let Some(renderer) = &mut self.renderer
                    && size.width != 0
                    && size.height != 0
                {
                    renderer.surface_config.width = size.width;
                    renderer.surface_config.height = size.height;
                    renderer
                        .surface
                        .configure(&renderer.device, &renderer.surface_config);
                }
            }

            WindowEvent::ScaleFactorChanged {
                scale_factor: _,
                inner_size_writer: _,
            } => {
                if let Some(renderer) = &mut self.renderer {
                    let size = renderer.window.inner_size();
                    renderer.surface_config.width = size.width;
                    renderer.surface_config.height = size.height;
                    renderer
                        .surface
                        .configure(&renderer.device, &renderer.surface_config);
                }
            }

            WindowEvent::KeyboardInput {
                device_id: _,
                event,
                is_synthetic: _,
            } => {
                if event.repeat {
                    return;
                }
                if let Some(_renderer) = &mut self.renderer {
                    match event.physical_key {
                        PhysicalKey::Code(code) => match event.state {
                            ElementState::Pressed => {
                                self.pressed_keys.insert(code as usize);

                                if code == KeyCode::Escape {
                                    event_loop.exit();
                                }
                            }
                            ElementState::Released => {
                                self.pressed_keys.remove(code as usize);
                            }
                        },
                        _ => {}
                    }
                }
            }
            _ => (),
        }

        //do this afterwards, to allow us to grab input first.
        //also on a resize event, need to pass off to egui AFTER we reconfigure the render pipeline,
        if let Some(renderer) = &mut self.renderer {
            let event_response = renderer.gui.state.on_window_event(&renderer.window, &event);
            if event_response.repaint {
                renderer.window.request_redraw();
            }

            if event_response.consumed {
                return;
            }
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .init();

    let event_loop = EventLoop::new().unwrap();
    //event_loop.set_control_flow(ControlFlow::Poll);
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;
    Ok(())
}
