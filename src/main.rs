use std::{borrow::Cow, sync::Arc};

use anyhow::Context;
use arboard::ImageData;
use image::{DynamicImage, ImageBuffer, Rgba};
use pixels::{wgpu::naga::proc::NameKey, Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::PhysicalSize,
    event::{ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{Key, NamedKey},
    window::{Window, WindowAttributes},
};

struct Drag {
    start: (f64, f64),
    end: Option<(f64, f64)>,
}

struct Selection {
    start: (f64, f64),
    end: (f64, f64),
}

struct AppContext {
    size: PhysicalSize<u32>,
    mouse_position: (f64, f64),
    current_drag: Option<Drag>,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    pixels: Pixels<'static>,
    window: Arc<Window>,
    selection: Option<Selection>,
}

impl AppContext {
    fn start_drag(&mut self) {
        self.current_drag = Some(Drag {
            start: self.mouse_position,
            end: None,
        });
    }

    fn end_drag(&mut self) {
        if let Some(drag) = &self.current_drag {
            self.selection = Some(Selection {
                start: drag.start,
                end: self.mouse_position,
            });
        }
        self.current_drag = None;
    }

    fn cancel_drag(&mut self) {
        self.current_drag = None;
    }

    fn update_mouse_position(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = (position.x, position.y);
        if let Some(drag) = &mut self.current_drag {
            drag.end = Some(self.mouse_position);
        }
    }

    fn save_selection_to_clipboard(&self) {
        let Some(selection) = &self.selection else {
            return;
        };
        let (start_x, start_y) = (selection.start.0 as u32, selection.start.1 as u32);
        let (end_x, end_y) = (selection.end.0 as u32, selection.end.1 as u32);

        let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
        let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));

        // Shave off a single pixel around the edge
        // let min_x = min_x.saturating_add(1);
        // let min_y = min_y.saturating_add(1);
        // let max_x = max_x.saturating_sub(1);
        // let max_y = max_y.saturating_sub(1);

        let width = (max_x - min_x) as usize;
        let height = (max_y - min_y) as usize;

        let mut image_data = Vec::new();
        for y in min_y..max_y {
            let row_start = (y * self.size.width + min_x) as usize * 4;
            let row_end = (y * self.size.width + max_x) as usize * 4;
            let row = &self.image.as_raw()[row_start..row_end];
            image_data.extend_from_slice(row);
        }

        let mut clipboard = arboard::Clipboard::new().unwrap();
        if width * height != image_data.len() / 4 {
            eprintln!(
                "Invalid selection size {:?} (w h p)",
                (width, height, image_data.len() / 4)
            );
            return;
        }
        let image_data = ImageData {
            width,
            height,
            bytes: Cow::Owned(image_data),
        };
        let _ = clipboard.set_image(image_data);
    }

    fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let monitor = xcap::Monitor::all()?
            .into_iter()
            .find(|m| m.is_primary())
            .with_context(|| "Could not get primary monitor")?;
        let image = monitor.capture_image()?;
        let size = PhysicalSize::new(monitor.width(), monitor.height());
        let window = Arc::new(
            event_loop.create_window(
                WindowAttributes::default()
                    .with_inner_size(size)
                    .with_resizable(false)
                    .with_decorations(false)
                    .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
            )?,
        );
        // let window = Arc::new(event_loop.create_window(WindowAttributes {
        //     inner_size: Some(size.into()),
        //     resizable: false,
        //     decorations: false,
        //     fullscreen: Some(winit::window::Fullscreen::Borderless(None)),
        //     ..Default::default()
        // })?);
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        Ok(Self {
            size,
            image,
            window,
            current_drag: None,
            pixels,
            mouse_position: (0.0, 0.0),
            selection: None,
        })
    }

    fn draw(&mut self) {
        let frame = self.pixels.frame_mut();
        frame.copy_from_slice(self.image.as_raw());

        if let Some(drag) = &self.current_drag {
            let start_x = drag.start.0 as u32;
            let start_y = drag.start.1 as u32;
            let end_x = drag.end.map_or(start_x, |end| end.0 as u32);
            let end_y = drag.end.map_or(start_y, |end| end.1 as u32);

            let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
            let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));

            for y in min_y..=max_y {
                for x in min_x..=max_x {
                    let pixel = &mut frame[(y * self.size.width + x) as usize * 4..];
                    pixel[0] = (pixel[0] as f32 * 0.5) as u8; // R
                    pixel[1] = (pixel[1] as f32 * 0.5) as u8; // G
                    pixel[2] = (pixel[2] as f32 * 0.5) as u8; // B
                    pixel[3] = 255; // A
                }
            }
        }
        if let Some(selection) = &self.selection {
            let start_x = selection.start.0 as u32;
            let start_y = selection.start.1 as u32;
            let end_x = selection.end.0 as u32;
            let end_y = selection.end.1 as u32;

            let (min_x, max_x) = (start_x.min(end_x), start_x.max(end_x));
            let (min_y, max_y) = (start_y.min(end_y), start_y.max(end_y));

            // Draw selection rectangle outline
            for x in min_x..=max_x {
                let top_index = (min_y * self.size.width + x) as usize * 4;
                let bottom_index = (max_y * self.size.width + x) as usize * 4;
                frame[top_index..top_index + 4].copy_from_slice(&[255, 0, 0, 255]); // Red color
                frame[bottom_index..bottom_index + 4].copy_from_slice(&[255, 0, 0, 255]);
            }
            for y in min_y..=max_y {
                let left_index = (y * self.size.width + min_x) as usize * 4;
                let right_index = (y * self.size.width + max_x) as usize * 4;
                frame[left_index..left_index + 4].copy_from_slice(&[255, 0, 0, 255]);
                frame[right_index..right_index + 4].copy_from_slice(&[255, 0, 0, 255]);
            }
        }

        self.pixels.render().unwrap();
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
        window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        let Some(context) = &mut self.context else {
            return;
        };

        if let WindowEvent::RedrawRequested = event {
            context.draw();
            context.window.request_redraw();
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
