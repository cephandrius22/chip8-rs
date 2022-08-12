#![deny(clippy::all)]
#![forbid(unsafe_code)]

use log::error;
use pixels::{Error, Pixels, SurfaceTexture};
use winit::dpi::LogicalSize;
use winit::event::{Event, VirtualKeyCode};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::WindowBuilder;
use winit_input_helper::WinitInputHelper;

use std::collections::HashMap;
use std::env;
use std::fs;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

struct Chip8 {
    memory: Vec<u8>,
    register: HashMap<u8, u8>,
    I: u16,
    sound_timer: u8,
    delay_timer: u8,
    pc: u16,
    sp: u8,
    stack: Vec<u8>,
    keys: [bool; 12],
    screen: [[u8; HEIGHT]; WIDTH]
}

fn main() -> Result<(), Error> {

    let file = std::env::args().nth(1).expect("please supply a file");
    let mut chip8 = Chip8::new();
    chip8.memory = fs::read(file).unwrap();

    env_logger::init();
    let event_loop = EventLoop::new();
    let mut input = WinitInputHelper::new();
    let window = {
        let size = LogicalSize::new(WIDTH as f64, HEIGHT as f64);
        WindowBuilder::new()
            .with_title("Hello Pixels")
            .with_inner_size(size)
            .with_min_inner_size(size)
            .build(&event_loop)
            .unwrap()
    };

    let mut pixels = {
        let window_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(window_size.width, window_size.height, &window);
        Pixels::new(WIDTH as u32, HEIGHT as u32, surface_texture)?
    };

    event_loop.run(move |event, _, control_flow| {
        // Draw the current frame
        if let Event::RedrawRequested(_) = event {
            chip8.draw(pixels.get_frame());
            if pixels
                .render()
                .map_err(|e| error!("pixels.render() failed: {}", e))
                .is_err()
            {
                *control_flow = ControlFlow::Exit;
                return;
            }
        }

        // Handle input events
        if input.update(&event) {
            // Close events
            if input.key_pressed(VirtualKeyCode::Escape) || input.quit() {
                *control_flow = ControlFlow::Exit;
                return;
            }

            // Resize the window
            if let Some(size) = input.window_resized() {
                pixels.resize_surface(size.width, size.height);
            }

            // Update internal state and request a redraw
            chip8.update();
            window.request_redraw();
        }
    });
}

impl Chip8 {
    /// Create a new `World` instance that can draw a moving box.
    fn new() -> Self {
        Self {
            memory: Vec::new(),
            register: HashMap::new(),
            I: 0,
            sound_timer: 0,
            delay_timer: 0,
            pc: 0x200,
            sp: 0,
            stack: Vec::new(),
            keys: [false; 12],
            screen: [[0; HEIGHT]; WIDTH]
        }
    }

    /// Update the `World` internal state; bounce the box around the screen.
    fn update(&mut self) {
        
    }

    /// Draw the `World` state to the frame buffer.
    ///
    /// Assumes the default texture format: `wgpu::TextureFormat::Rgba8UnormSrgb`
    fn draw(&self, frame: &mut [u8]) {
        for (i, pixel) in frame.chunks_exact_mut(4).enumerate() {
            let _x = (i % WIDTH as usize) as i16;
            let _y = (i / WIDTH as usize) as i16;

            let rgba = [0x5e, 0x48, 0xe8, 0xff];

            pixel.copy_from_slice(&rgba);
        }
    }
}
