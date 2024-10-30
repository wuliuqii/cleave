use anyhow::Context;
use arboard::ImageData;
use image::{GenericImageView, ImageBuffer, Rgba};
// use pixels::{Pixels, SurfaceTexture};
use winit::{
    dpi::PhysicalSize,
    window::{Window, WindowAttributes},
};

use crate::{graphics_bundle::GraphicsBundle, graphics_impl::Graphics};

struct Drag {
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

struct Selection {
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

pub struct AppContext {
    size: PhysicalSize<u32>,
    mouse_position: (f64, f64),
    current_drag: Option<Drag>,
    image: ImageBuffer<Rgba<u8>, Vec<u8>>,
    // pixels: Pixels<'static>,
    total_time: f32,
    last_frame: std::time::Instant,
    graphics: Graphics<Window>,
    bundle: GraphicsBundle,
    selection: Option<Selection>,
}

impl AppContext {
    pub fn start_drag(&mut self) {
        self.current_drag = Some(Drag {
            start: self.mouse_position,
            end: None,
        });
    }

    pub fn end_drag(&mut self) {
        if let Some(drag) = &self.current_drag {
            self.selection = Some(Selection {
                start: drag.start,
                end: self.mouse_position,
            });
        }
        self.current_drag = None;
    }

    pub fn cancel_drag(&mut self) {
        self.current_drag = None;
    }

    pub fn update_mouse_position(&mut self, position: winit::dpi::PhysicalPosition<f64>) {
        self.mouse_position = (position.x, position.y);
        if let Some(drag) = &mut self.current_drag {
            drag.end = Some(self.mouse_position);
        }
    }

    pub fn save_selection_to_clipboard(&self) {
        let Some(selection) = &self.selection else {
            return;
        };

        let ((min_x, min_y), (max_x, max_y)) = selection.coords();

        let (width, height) = selection.dimensions();
        let width = width as usize;
        let height = height as usize;

        let mut image_data = Vec::new();
        for y in min_y..max_y {
            let row_start = (y * self.size.width + min_x) as usize * 4;
            let row_end = (y * self.size.width + max_x) as usize * 4;
            let row = &self.image.as_raw()[row_start..row_end];
            image_data.extend_from_slice(row);
        }

        let mut clipboard = arboard::Clipboard::new().unwrap();
        if selection.area() as usize != image_data.len() / 4 {
            eprintln!(
                "Invalid selection size {:?} (w h p)",
                (width, height, image_data.len() / 4)
            );
            return;
        }
        let image_data = ImageData {
            width,
            height,
            bytes: std::borrow::Cow::Owned(image_data),
        };
        let _ = clipboard.set_image(image_data);
    }

    pub fn new(event_loop: &winit::event_loop::ActiveEventLoop) -> anyhow::Result<Self> {
        let monitor = xcap::Monitor::all()?
            .into_iter()
            .find(|m| m.is_primary())
            .with_context(|| "Could not get primary monitor")?;
        let image = monitor.capture_image()?;
        let size = PhysicalSize::new(monitor.width(), monitor.height());
        let window = event_loop.create_window(
            WindowAttributes::default()
                .with_inner_size(size)
                .with_resizable(false)
                .with_decorations(false)
                .with_fullscreen(Some(winit::window::Fullscreen::Borderless(None))),
        )?;

        let graphics = Graphics::new(window, size);
        let graphics = pollster::block_on(graphics)?;

        let bundle = GraphicsBundle::new(
            image.clone().into(),
            &graphics.device,
            &graphics.queue,
            wgpu::PrimitiveTopology::TriangleStrip,
            graphics.config.format,
        );

        // let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        // let pixels = Pixels::new(size.width, size.height, surface_texture)?;

        Ok(Self {
            size,
            image,
            bundle,
            total_time: 0.0,
            last_frame: std::time::Instant::now(),
            current_drag: None,
            // window,
            graphics,
            mouse_position: (0.0, 0.0),
            selection: None,
        })
    }

    pub fn draw(&mut self) {
        let time = self.last_frame.elapsed().as_secs_f32();
        self.total_time += time;
        self.last_frame = std::time::Instant::now();
        self.bundle.update_buffers(self.total_time, &self.graphics.queue);

        let mut pass = match self.graphics.render() {
            Ok(pass) => pass,
            Err(err) => {
                eprintln!("Error rendering frame: {:?}", err);
                return;
            }
        };

        self.bundle.draw(&mut pass);
        pass.finish();
        self.graphics.request_redraw();
    }

    pub fn window_id(&self) -> winit::window::WindowId {
        self.graphics.id()
    }
    
    pub fn destroy(&self) {
      self.graphics.window.set_minimized(true);
    }
}

fn draw_rectangle_outline(
    img_width: u32,
    min_x: u32,
    min_y: u32,
    max_x: u32,
    max_y: u32,
    frame: &mut [u8],
) {
    for x in min_x..=max_x {
        let top_index = (min_y * img_width + x) as usize * 4;
        let bottom_index = (max_y * img_width + x) as usize * 4;
        frame[top_index..top_index + 4].copy_from_slice(&[255, 0, 0, 255]); // Red color
        frame[bottom_index..bottom_index + 4].copy_from_slice(&[255, 0, 0, 255]);
    }
    for y in min_y..=max_y {
        let left_index = (y * img_width + min_x) as usize * 4;
        let right_index = (y * img_width + max_x) as usize * 4;
        frame[left_index..left_index + 4].copy_from_slice(&[255, 0, 0, 255]);
        frame[right_index..right_index + 4].copy_from_slice(&[255, 0, 0, 255]);
    }
}
