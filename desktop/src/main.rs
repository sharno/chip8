use core::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};
use ggez::conf::{WindowMode, WindowSetup};
use ggez::event::{self, EventHandler};
use ggez::graphics::{self, Color, DrawMode, DrawParam, Mesh, MeshBuilder, Rect};
use ggez::input::keyboard::KeyCode;
use ggez::{Context, GameResult};
use std::{env, fs::File, io::Read};

const SCALE: f32 = 15.0;
const WINDOW_WIDTH: f32 = (SCREEN_WIDTH as f32) * SCALE;
const WINDOW_HEIGHT: f32 = (SCREEN_HEIGHT as f32) * SCALE;
const TICKS_PER_FRAME: usize = 10;

struct EmulatorFrontend {
    emu: Emu,
}

impl EmulatorFrontend {
    fn new(rom_path: &str) -> Self {
        let mut emu = Emu::new();
        let mut rom = File::open(rom_path).expect("The rom doesn't exist");
        let mut buffer: Vec<u8> = Vec::new();
        rom.read_to_end(&mut buffer)
            .expect("Error while reading the rom file");
        emu.load(&buffer);
        EmulatorFrontend { emu }
    }

    // Map keycode to CHIP-8 key index
    fn key_to_chip8(&self, key: KeyCode) -> Option<usize> {
        match key {
            // CHIP-8 keyboard layout:
            // 1 2 3 C
            // 4 5 6 D
            // 7 8 9 E
            // A 0 B F
            KeyCode::Key1 => Some(0x1),
            KeyCode::Key2 => Some(0x2),
            KeyCode::Key3 => Some(0x3),
            KeyCode::Key4 => Some(0xC),
            KeyCode::Q => Some(0x4),
            KeyCode::W => Some(0x5),
            KeyCode::E => Some(0x6),
            KeyCode::R => Some(0xD),
            KeyCode::A => Some(0x7),
            KeyCode::S => Some(0x8),
            KeyCode::D => Some(0x9),
            KeyCode::F => Some(0xE),
            KeyCode::Z => Some(0xA),
            KeyCode::X => Some(0x0),
            KeyCode::C => Some(0xB),
            KeyCode::V => Some(0xF),
            _ => None,
        }
    }
}

impl EventHandler for EmulatorFrontend {
    fn update(&mut self, ctx: &mut Context) -> GameResult {
        // Handle keyboard input
        let keys = [
            KeyCode::Key1,
            KeyCode::Key2,
            KeyCode::Key3,
            KeyCode::Key4,
            KeyCode::Q,
            KeyCode::W,
            KeyCode::E,
            KeyCode::R,
            KeyCode::A,
            KeyCode::S,
            KeyCode::D,
            KeyCode::F,
            KeyCode::Z,
            KeyCode::X,
            KeyCode::C,
            KeyCode::V,
        ];

        for &key in keys.iter() {
            if let Some(chip8_key) = self.key_to_chip8(key) {
                let is_pressed = ctx.keyboard.is_key_pressed(key);
                self.emu.key_press(chip8_key, is_pressed);
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            self.emu.tick();
        }
        self.emu.tick_timers();
        Ok(())
    }

    fn draw(&mut self, ctx: &mut Context) -> GameResult {
        let mut canvas = graphics::Canvas::from_frame(ctx, Color::BLACK);
        let screen_buffer = self.emu.get_display();
        let mut mesh_builder = MeshBuilder::new();
        for (i, pixel) in screen_buffer.iter().enumerate() {
            if *pixel {
                let x = (i % SCREEN_WIDTH) as f32 * SCALE;
                let y = (i / SCREEN_WIDTH) as f32 * SCALE;
                let rect = Rect::new(x, y, SCALE, SCALE);
                mesh_builder.rectangle(DrawMode::fill(), rect, Color::WHITE)?;
            }
        }
        let mesh_data = mesh_builder.build();
        let mesh = Mesh::from_data(ctx, mesh_data);
        canvas.draw(&mesh, DrawParam::default());
        canvas.finish(ctx)?;
        Ok(())
    }
}

fn main() -> GameResult {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Please specify the rom file");
        return Ok(());
    }
    let (ctx, event_loop) = ggez::ContextBuilder::new("chip8", "author")
        .window_setup(WindowSetup::default().title("CHIP-8 Emulator"))
        .window_mode(WindowMode::default().dimensions(WINDOW_WIDTH, WINDOW_HEIGHT))
        .build()?;
    let frontend = EmulatorFrontend::new(&args[1]);
    event::run(ctx, event_loop, frontend)
}
