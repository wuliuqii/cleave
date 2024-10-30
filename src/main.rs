use winit::{
    application::ApplicationHandler,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
};

mod context;
mod graphics_bundle;
mod graphics_impl;
mod instance;
mod texture;
mod vertex;
use context::AppContext;


pub struct Drag {
  start: (f64, f64),
  end: Option<(f64, f64)>,
}

impl Drag {
  fn coords(&self) -> Option<((u32, u32), (u32, u32))> {
      let end = self.end?;
      let (start_x, start_y) = (self.start.0 as u32, self.start.1 as u32);
      let (end_x, end_y) = (end.0 as u32, end.1 as u32);

      let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
      let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));
      Some(((min_x, min_y), (max_x, max_y)))
  }
}

pub struct Selection {
  start: (f64, f64),
  end: (f64, f64),
}

impl Selection {
  fn dimensions(&self) -> (f64, f64) {
      let width = (self.end.0 - self.start.0).abs();
      let height = (self.end.1 - self.start.1).abs();
      (width, height)
  }

  fn area(&self) -> f64 {
      let (width, height) = self.dimensions();
      width * height
  }

  // fn aspect_ratio(&self) -> f64 {
  //     let (width, height) = self.dimensions();
  //     width / height
  // }

  // fn center(&self) -> (f64, f64) {
  //     let x = (self.start.0 + self.end.0) / 2.0;
  //     let y = (self.start.1 + self.end.1) / 2.0;
  //     (x, y)
  // }

  fn coords(&self) -> ((u32, u32), (u32, u32)) {
      let (start_x, start_y) = (self.start.0 as u32, self.start.1 as u32);
      let (end_x, end_y) = (self.end.0 as u32, self.end.1 as u32);

      let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
      let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));
      ((min_x, min_y), (max_x, max_y))
  }
}

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
        id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(context) = &mut self.context else {
            return;
        };
        if id != context.window_id() {
            return;
        }

        match event {
            WindowEvent::RedrawRequested => {
                context.draw();
            }
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
                context.destroy();
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
                context.hide_window();
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
