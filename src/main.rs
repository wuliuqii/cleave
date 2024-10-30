use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

mod context;
use context::AppContext;

struct App {
    context: Option<AppContext>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &winit::event_loop::ActiveEventLoop) {
        let context = AppContext::new(event_loop).expect("Could not start context");
        self.context = Some(context);
    }

    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(context) = &mut self.context else {
            return;
        };

        if let WindowEvent::RedrawRequested = event {
            context.draw();
            return;
        }

        match event {
            WindowEvent::CursorMoved { position, .. } => {
                context.update_mouse_position(position);
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Escape),
                        ..
                    },
                ..
            } => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        logical_key: Key::Named(NamedKey::Space),
                        ..
                    },
                ..
            } => {
                context.save_selection_to_clipboard();
                event_loop.exit();
            }
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                context.start_drag();
            }
            WindowEvent::MouseInput {
                state: ElementState::Released,
                button: MouseButton::Left,
                ..
            } => {
                context.end_drag();
            }
            WindowEvent::MouseInput {
                button: MouseButton::Right,
                ..
            } => {
                context.cancel_drag();
            }
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            _ => {}
        }
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App { context: None };
    let event_loop = winit::event_loop::EventLoop::new()?;
    event_loop.run_app(&mut app)?;
    Ok(())
}
