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

use std::io;
use std::io::prelude::*;
use std::fs::File;

const WIDTH: usize = 320;
const HEIGHT: usize = 240;

struct Chip8 {
    memory: Vec<u8>,
    registers: HashMap<u8, u8>,
    I: u16,
    sound_timer: u8,
    delay_timer: u8,
    pc: u16,
    sp: u8,
    stack: Vec<u16>,
    keys: [bool; 12],
    screen: [[u8; HEIGHT]; WIDTH]
}

fn main() -> Result<(), Error> {

    let filename = std::env::args().nth(1).expect("please supply a file");
    let mut chip8 = Chip8::new();

    chip8.load_file(&filename).expect("Failed to open {filename}");
    chip8.dump_memory();

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

    
    /// Load a chip8 file into memory.
    fn load_file(&mut self, filename: &str) -> io::Result<()> {
        let mut f = File::open(filename)?;
        f.read_to_end(&mut self.memory)?;
        Ok(())
    }

    fn dump_memory(&self) {
        for (addr, mem) in self.memory.chunks(2).enumerate() {
            println!("{} {}", mem[0], mem[1]);
            let instruction = (mem[0] as u16) | ((mem[1] as u16) << 8);
            println!("instruction: {}", instruction);
            println!("{:#04x}: {:#04x}", addr, instruction);
        }
    }

    fn clear_display(&self) {
        self.screen = [[0; HEIGHT]; WIDTH]
    }

    fn execute_instruction(&mut self) {
        let index = self.pc as usize;
        let opcode: u16 = ((self.memory[index + 1] as u16) << 8) | (self.memory[index] as u16);
        let msb = (opcode >> 8) as u8;
        let lsb = (opcode & 0xFF) as u8;

        self.pc += 2;

        match (msb >> 4) {
            0x0 => {
                match opcode {
                    0x00E0 => self.clear_display(),
                    0x00EE => self.pc = self.stack.pop().unwrap(),
                }
            },
            0x1 => self.pc = (opcode & 0x0FFF),
            0x2 => {
                let call_location = (opcode & 0x0FFF);
                self.stack.push(self.pc);
                self.pc = call_location;
            },
            0x3 => {
                let reg = ((opcode >> 8) & 0x0F) as u8;
                let kk = (opcode & 0xFF) as u8;
                if kk == *self.registers.get(&reg).unwrap() {
                    self.pc += 2;
                }
            },
            0x4 => {
                let reg = ((opcode >> 8) & 0x0F) as u8;
                let kk = (opcode & 0xFF) as u8;
                if kk != *self.registers.get(&reg).unwrap() {
                    self.pc += 2;
                }
            },
            0x5 => {
                let x = ((opcode >> 8) & 0x0F) as u8;
                let y = ((opcode >> 4) & 0x0F) as u8;
                if *self.registers.get(&x).unwrap() == *self.registers.get(&y).unwrap() {
                    self.pc += 2;
                }
            },
            0x6 => {
                let reg = ((opcode >> 8) & 0x0F) as u8;
                let kk = (opcode & 0xFF) as u8;
                self.registers.insert(reg, kk);
            },
            0x7 => {
                let reg = ((opcode >> 8) & 0x0F) as u8;
                let kk = (opcode & 0xFF) as u8;
                let val = self.registers.get(&reg).unwrap();
                self.registers.insert(reg, val + kk);
            },
            0x8 => {
                let x = ((opcode >> 8) & 0x0F) as u8;
                let y = ((opcode >> 4) & 0x0F) as u8;
                let x_val = *self.registers.get(&y).unwrap();
                let y_val = *self.registers.get(&y).unwrap();
                match (opcode & 0xF) {
                    0 => {self.registers.insert(x, y_val);},
                    1 => {self.registers.insert(x, x_val  | y_val);},
                    2 => {self.registers.insert(x, x_val & y_val);},
                    3 => {self.registers.insert(x, x_val ^ y_val);},
                }
            },
            0x9 => 0,
        }
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
