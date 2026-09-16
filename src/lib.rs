#![allow(dead_code)]

use bit_set::BitSet;
use bytemuck::bytes_of;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen_futures::wasm_bindgen::prelude::wasm_bindgen;
use std::error::Error;
use std::f32::consts::PI;
use winit::application::ApplicationHandler;
use winit::dpi::{LogicalSize, Size};
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::KeyCode::{self};
use winit::keyboard::PhysicalKey::{self};
use winit::window::{Window, WindowId};

#[cfg(not(target_arch = "wasm32"))]
use smol::future;

#[cfg(target_arch = "wasm32")]
use futures::channel::oneshot;

#[cfg(target_arch = "wasm32")]
use winit::platform::web::WindowAttributesExtWebSys;

mod camera;
mod geometry;
mod gui;
mod move_controller;
mod renderer;
mod util;
mod shaders;

use camera::*;
use move_controller::*;
use renderer::*;

//which item is currently controllable with wasd/arrow keys.
#[derive(Debug)]
enum ActiveMoveItem {
    Camera,
    Cube,
}

pub struct App {
    renderer: Option<Renderer>,
    #[cfg(target_arch = "wasm32")]
    renderer_rx:Option<oneshot::Receiver<Renderer>>,

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
            #[cfg(target_arch = "wasm32")]
            renderer_rx: None,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {

        log::info!("App resumed");
        if let None = self.renderer {

            let win_attribs = Window::default_attributes().with_inner_size(Size::Logical(
                    LogicalSize {
                        width: 800.0,
                        height: 800.0,
                    },
                ));
            
            #[cfg(target_arch = "wasm32")]
            let win_attribs = win_attribs.with_append(true);

            let window = event_loop.create_window(win_attribs).unwrap();

            

            #[cfg(target_arch = "wasm32")] {
                let (tx,rx) = oneshot::channel::<Renderer>();
                self.renderer_rx = Some(rx);
                wasm_bindgen_futures::spawn_local(async move {
                    let renderer = Renderer::new(window).await;
                    //TODO this is becoming messy, we do this in two places
                    // renderer.queue.write_buffer(
                    //     &renderer.camera_transform_buffer,
                    //     0,
                    //     bytes_of(&self.camera.update(&self.pressed_keys)),
                    // );
                    tx.send(renderer).unwrap_or_else(|_| {
                        log::info!("failed to send renderer over oneshot channel rx end dropped)");
                    });
                });
            }

            #[cfg(not(target_arch = "wasm32"))]
            {
                let renderer = smol::block_on(Renderer::new(window));
                //TODO this is becoming messy, we do this in two places
                renderer.queue.write_buffer(
                    &renderer.camera_transform_buffer,
                    0,
                    bytes_of(&self.camera.update(&self.pressed_keys)),
                );
                self.renderer.replace(renderer);
            }
            
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {

         #[cfg(target_arch = "wasm32")] {
            if let Some(rx) = &mut self.renderer_rx {
                if let Ok(Some(renderer)) = rx.try_recv(){
                    self.renderer = Some(renderer);
                }
            }
         }

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
                log::info!("window resized to ({},{})",size.width,size.height);
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
                if let Some(renderer) = &mut self.renderer 
                {
                    let size = renderer.window.inner_size();
                    if size.width != 0 && size.height != 0 {
                        renderer.surface_config.width = size.width;
                        renderer.surface_config.height = size.height;
                        renderer.surface.configure(&renderer.device, &renderer.surface_config);
                    }
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


#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_wasm() {
    use winit::platform::web::EventLoopExtWebSys;

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("Couldn't init logger");

    log::info!("Hi There from UnladenCoconut!");

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_poll_strategy(winit::platform::web::PollStrategy::Scheduler);

    let app = App::default();
    event_loop.spawn_app(app);
}