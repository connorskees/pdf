use winit::{
    dpi::LogicalSize,
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};

use crate::data_structures::Matrix;

use self::state::State;

use super::Renderable;

mod state;

pub async fn run(to_render: &[Renderable], width: f32, height: f32) {
    env_logger::init();

    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new()
        .with_inner_size(LogicalSize::new(width, height))
        .build(&event_loop)
        .unwrap();

    // State::new uses async code, so we're going to wait for it to finish
    let mut state = State::new(&window).await;
    let mut surface_configured = false;

    let mut is_ctrl_pressed = false;
    // let mut transform = Matrix::new_translation(-1.75, -20.5) * Matrix::new_scale(22.0, 22.0);
    let mut transform = Matrix::identity();

    event_loop
        .run(move |event, control_flow| {
            match event {
                Event::WindowEvent {
                    ref event,
                    window_id,
                } if window_id == state.window().id() => {
                    if !state.input(event) {
                        match event {
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Pressed,
                                        physical_key: PhysicalKey::Code(KeyCode::SuperLeft),
                                        ..
                                    },
                                ..
                            } => {
                                is_ctrl_pressed = true;
                            }
                            WindowEvent::KeyboardInput {
                                event:
                                    KeyEvent {
                                        state: ElementState::Released,
                                        physical_key: PhysicalKey::Code(KeyCode::SuperLeft),
                                        ..
                                    },
                                ..
                            } => {
                                is_ctrl_pressed = false;
                            }
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
                                surface_configured = true;
                                state.resize(*physical_size);
                            }
                            WindowEvent::MouseWheel { delta, .. } => {
                                let x: f32;
                                let y: f32;

                                match delta {
                                    MouseScrollDelta::LineDelta(_, _) => todo!(),
                                    MouseScrollDelta::PixelDelta(pos) => {
                                        x = pos.x as f32;
                                        y = pos.y as f32;
                                    }
                                }

                                if is_ctrl_pressed {
                                    let y_pct = y / (height / 2.0);
                                    let scale = (1.0 + y_pct).min(1.05).max(0.95);
                                    transform *= Matrix::new_scale(scale, scale);
                                    transform *= Matrix::new_translation(
                                        ((1.0 - scale) * (width / 2.0)) / width,
                                        ((1.0 - scale) * (height / 2.0)) / height,
                                    );
                                } else {
                                    transform *= Matrix::new_translation(x / width, -y / height);
                                }
                            }
                            WindowEvent::RedrawRequested => {
                                // This tells winit that we want another frame after this one
                                state.window().request_redraw();

                                if !surface_configured {
                                    return;
                                }

                                state.update();
                                match state.render(to_render, width, height, transform) {
                                    Ok(_) => {}
                                    // Reconfigure the surface if it's lost or outdated
                                    Err(
                                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                                    ) => state.resize(state.size),
                                    // The system is out of memory, we should probably quit
                                    Err(wgpu::SurfaceError::OutOfMemory) => {
                                        log::error!("OutOfMemory");
                                        control_flow.exit();
                                    }

                                    // This happens when the a frame takes too long to present
                                    Err(wgpu::SurfaceError::Timeout) => {
                                        log::warn!("Surface timeout")
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        })
        .unwrap();
}
