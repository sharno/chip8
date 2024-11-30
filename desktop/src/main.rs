use core::{Emu, SCREEN_HEIGHT, SCREEN_WIDTH};
use std::{env, fs::File, io::Read, ops::Index};

use sdl2::{pixels::Color, rect::Rect, render::Canvas, video::Window};

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

const TICKS_PER_FRAME: usize = 10;

fn main() {
    let args: Vec<_> = env::args().collect();
    if args.len() < 2 {
        println!("Please specify the rom file");
        return;
    }

    let sdl_context = sdl2::init().expect("Failed to initialize SDL2");
    let mut event_pump = sdl_context.event_pump().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("CHIP-8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    canvas.clear();
    canvas.present();

    let mut emu = Emu::new();
    let mut rom = File::open(&args[1]).expect("The rom doesn't exist");
    let mut buffer: Vec<u8> = Vec::new();
    rom.read_to_end(&mut buffer)
        .expect("Error while reading the rom file");
    emu.load(&buffer);

    'gameloop: loop {
        for event in event_pump.poll_event() {}

        for _ in 0..TICKS_PER_FRAME {
            emu.tick();
        }
        emu.tick_timers();
        draw_screen(&mut emu, &mut canvas);
    }
}

fn draw_screen(emu: &mut Emu, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    let screen_buffer = emu.get_screen();
    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in screen_buffer.iter().enumerate() {
        if *pixel {
            let x = (i % SCREEN_WIDTH) as u32;
            let y = (i / SCREEN_HEIGHT) as u32;
            let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
            canvas.fill_rect(rect).unwrap();
        }
    }
}
