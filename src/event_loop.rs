use winit::{
    event::{ElementState, Event, KeyEvent, WindowEvent},
    event_loop::ActiveEventLoop,
};

use crate::State;

#[cfg(not(target_family = "wasm"))]
type EventTypeUsed = winit::event::Event<()>;

pub fn handle_event_loop(
    event: &EventTypeUsed,
    state: &mut State,
    event_loop_window_target: &ActiveEventLoop,
) {
    match event {
        &Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => match event {
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key:
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowDown),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.change_rule(true);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowUp),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.change_rule(false);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key:
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowLeft),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.set_initial_density(state.initial_density - 1);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key:
                            winit::keyboard::Key::Named(winit::keyboard::NamedKey::ArrowRight),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.set_initial_density(state.initial_density + 1);
            }
            WindowEvent::KeyboardInput {
                event:
                    winit::event::KeyEvent {
                        logical_key: winit::keyboard::Key::Character(c),
                        state: winit::event::ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                if c == "f" || c == "F" {
                    #[cfg(not(target_family = "wasm"))]
                    {
                        if state.window.fullscreen().is_some() {
                            state.window.set_fullscreen(None);
                        } else {
                            state
                                .window
                                .set_fullscreen(Some(winit::window::Fullscreen::Borderless(None)));
                        }
                    }
                } else if c == "r" || c == "R" {
                    state.reset();
                } else if c == "q" || c == "Q" {
                    state.set_generations_per_second(state.generations_per_second - 1);
                } else if c == "<" {
                    state.set_initial_density(state.initial_density - 1);
                } else if c == ">" {
                    state.set_initial_density(state.initial_density + 1);
                } else if c == "w" || c == "W" {
                    state.set_generations_per_second(state.generations_per_second + 1);
                } else if c == "-" && state.cells_width < 2048 {
                    let current_idx = State::ELIGIBLE_SIZES
                        .iter()
                        .position(|&s| s == state.cells_width)
                        .unwrap();
                    let new_size = State::ELIGIBLE_SIZES[current_idx + 1];
                    state.reset_with_cells_width(new_size, new_size);
                } else if c == "+" && state.cells_width > 64 {
                    let current_idx = State::ELIGIBLE_SIZES
                        .iter()
                        .position(|&s| s == state.cells_width)
                        .unwrap();
                    let new_size = State::ELIGIBLE_SIZES[current_idx - 1];
                    state.reset_with_cells_width(new_size, new_size);
                }
            }
            #[cfg(not(target_family = "wasm"))]
            #[cfg(not(target_arch = "android"))]
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Escape),
                        ..
                    },
                ..
            } => event_loop_window_target.exit(),
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        logical_key: winit::keyboard::Key::Named(winit::keyboard::NamedKey::Space),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => {
                state.toggle_pause();
            }
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            }
            WindowEvent::RedrawRequested => match state.render() {
                Ok(_) => {}
                Err(wgpu::SurfaceError::Lost) => state.resize(state.size),
                Err(wgpu::SurfaceError::OutOfMemory) => event_loop_window_target.exit(),
                Err(e) => log::error!("{:?}", e),
            },
            _ => {}
        },
        Event::AboutToWait => {
            state.window.request_redraw();
        }
        Event::Resumed => {
            state.last_time = instant::Instant::now();
        }
        _ => {}
    }
}
