mod chip8;

use std::sync::Arc;

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowAttributes, WindowId},
};

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("Crisp-8")
                        .with_visible(true)
                        .with_active(true)
                        .with_inner_size(LogicalSize::new(640, 320)),
                )
                .expect("Failed to create window"),
        );

        let size = window.inner_size();
        let surface_texture = SurfaceTexture::new(size.width, size.height, window.clone());
        let pixels = Pixels::new(64, 32, surface_texture).expect("Failed to create pixel buffer");

        self.window = Some(window);
        self.pixels = Some(pixels);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::Resized(size) => {
                if size.width == 0 || size.height == 0 {
                    return;
                }
                if let Some(pixels) = self.pixels.as_mut() {
                    pixels
                        .resize_surface(size.width, size.height)
                        .expect("Failed to resize surface");
                }
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
            }
            WindowEvent::RedrawRequested => {
                let (Some(pixels), Some(window)) = (self.pixels.as_mut(), self.window.as_ref())
                else {
                    return;
                };

                for pixel in pixels.frame_mut().chunks_exact_mut(4) {
                    pixel.copy_from_slice(&[0x00, 0x00, 0x00, 0xff]);
                }

                if let Err(err) = pixels.render() {
                    eprintln!("Failed to render frame: {err}");
                    event_loop.exit();
                    return;
                }

                window.request_redraw();
            }
            _ => (),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        pixels: None,
    };
    event_loop.run_app(&mut app)?;

    Ok(())
}
