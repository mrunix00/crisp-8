mod chip8;

use std::{
    fs::{self},
    sync::Arc,
};

use pixels::{Pixels, SurfaceTexture};
use winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::{ElementState, WindowEvent},
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowAttributes, WindowId},
};

use crate::chip8::Chip8VM;

fn keypad_key(code: KeyCode) -> Option<u8> {
    Some(match code {
        KeyCode::Digit1 => 0x1,
        KeyCode::Digit2 => 0x2,
        KeyCode::Digit3 => 0x3,
        KeyCode::Digit4 => 0xc,
        KeyCode::KeyQ => 0x4,
        KeyCode::KeyW => 0x5,
        KeyCode::KeyE => 0x6,
        KeyCode::KeyR => 0xd,
        KeyCode::KeyA => 0x7,
        KeyCode::KeyS => 0x8,
        KeyCode::KeyD => 0x9,
        KeyCode::KeyF => 0xe,
        KeyCode::KeyZ => 0xa,
        KeyCode::KeyX => 0xb,
        KeyCode::KeyC => 0xf,
        KeyCode::KeyV => 0x0,
        _ => return None,
    })
}

struct App {
    window: Option<Arc<Window>>,
    pixels: Option<Pixels<'static>>,
    chip8: Chip8VM,
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
            WindowEvent::KeyboardInput {
                event,
                is_synthetic: false,
                ..
            } => {
                let PhysicalKey::Code(code) = event.physical_key else {
                    return;
                };
                let Some(key) = keypad_key(code) else {
                    return;
                };
                match event.state {
                    ElementState::Pressed => self.chip8.key_down(key),
                    ElementState::Released => self.chip8.key_up(key),
                }
            }
            WindowEvent::RedrawRequested => {
                self.chip8.tick();

                let (Some(pixels), Some(window)) = (self.pixels.as_mut(), self.window.as_ref())
                else {
                    return;
                };

                for (pixel, &on) in pixels
                    .frame_mut()
                    .as_chunks_mut::<4>()
                    .0
                    .iter_mut()
                    .zip(self.chip8.display.iter())
                {
                    let color = if on != 0 {
                        [0xff, 0xff, 0xff, 0xff]
                    } else {
                        [0x00, 0x00, 0x00, 0xff]
                    };
                    pixel.copy_from_slice(&color);
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
    let args: Vec<String> = std::env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage: {} [FILE]", args[0]);
        std::process::exit(1);
    }

    let rom_file = match fs::read(&args[1]) {
        Ok(file) => file,
        Err(err) => {
            eprintln!("Failed to read ROM file: {}", err);
            std::process::exit(1);
        }
    };

    if rom_file.len() > 4096 {
        eprintln!("ROM file is too large");
        std::process::exit(1);
    }

    let vm = Chip8VM::new(&rom_file);

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        pixels: None,
        chip8: vm,
    };
    event_loop.run_app(&mut app)?;

    Ok(())
}
