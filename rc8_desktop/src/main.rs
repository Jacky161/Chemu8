mod audio;

use std::fs;

use audio::SineWave;
use rc8_core::Chip8;

use clap::Parser;

use sdl2::Sdl;
use sdl2::audio::{AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::Canvas;
use sdl2::video::Window;

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;
const TICKS_PER_FRAME: usize = 10;
const FRAMERATE: u32 = 60;

const SCALE: u32 = 15;
const WINDOW_WIDTH: u32 = (SCREEN_WIDTH as u32) * SCALE;
const WINDOW_HEIGHT: u32 = (SCREEN_HEIGHT as u32) * SCALE;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Path to the rom to load.
    rom_path: String,

    /// Enable all Cowgod quirks. Some games rely on an inaccurate implementation of
    /// certain instructions and won't run properly without this being enabled.
    #[arg(long, default_value_t = false)]
    quirk_cowgod: bool,
}

fn main() {
    // Read ROM file from CLI args
    let args = Args::parse();

    let mut chip8 = Chip8::new(args.quirk_cowgod, args.quirk_cowgod, args.quirk_cowgod, args.quirk_cowgod);
    let rom_data = fs::read(args.rom_path).expect("Failed to read ROM!");
    chip8.load(&rom_data);

    // Setup SDL and Display
    let sdl_context = sdl2::init().unwrap();
    let audio_device = setup_audio(&sdl_context);
    let (fps_manager, canvas) = setup_display(&sdl_context);

    run_loop(chip8, &sdl_context, canvas, fps_manager, audio_device);
}

fn setup_display(sdl_context: &Sdl) -> (FPSManager, Canvas<Window>) {
    // SETUP DISPLAY
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("RC8 Emulator", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();

    // Limit framerate
    let mut fps_manager = FPSManager::new();
    fps_manager
        .set_framerate(FRAMERATE)
        .expect("Failed to limit framerate!");

    canvas.clear();
    canvas.present();

    return (fps_manager, canvas);
}

fn setup_audio(sdl_context: &Sdl) -> AudioDevice<SineWave> {
    let audio_subsystem = sdl_context.audio().unwrap();

    let desired_spec = AudioSpecDesired {
        freq: Some(44100),
        channels: Some(1), // mono
        samples: None,     // default sample size
    };

    let device = audio_subsystem
        .open_playback(None, &desired_spec, |spec| {
            // initialize the audio callback
            SineWave {
                phase_inc: 1000.0 / spec.freq as f32,
                phase: 0.0,
                volume: 0.25,
            }
        })
        .unwrap();

    return device;
}

fn run_loop(
    mut chip8: Chip8,
    sdl_context: &Sdl,
    mut canvas: Canvas<Window>,
    mut fps_manager: FPSManager,
    audio_device: AudioDevice<SineWave>,
) {
    let mut event_pump = sdl_context.event_pump().unwrap();

    'gameloop: loop {
        // Handle SDL Events
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyUp {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'gameloop,
                Event::KeyDown {
                    keycode: Some(key), ..
                } => {
                    if let Some(c8key) = keycode_to_c8(key) {
                        chip8.set_key(c8key, true);
                    }
                }
                Event::KeyUp {
                    keycode: Some(key), ..
                } => {
                    if let Some(c8key) = keycode_to_c8(key) {
                        chip8.set_key(c8key, false);
                    }
                }
                _ => {}
            }
        }

        for _ in 0..TICKS_PER_FRAME {
            chip8.tick();
        }

        if chip8.tick_timers() {
            audio_device.resume();
        } else {
            audio_device.pause();
        }

        draw_screen(&chip8, &mut canvas);

        // Delay to maintain framerate
        chip8.notify_vblank();
        fps_manager.delay();
    }
}

fn draw_screen(chip8: &Chip8, canvas: &mut Canvas<Window>) {
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();

    canvas.set_draw_color(Color::RGB(255, 255, 255));
    for (i, pixel) in chip8.screen.iter().enumerate() {
        // Don't do anything if it's black
        if !*pixel {
            continue;
        }

        // Convert 1D index into (x,y)
        let x = (i % SCREEN_WIDTH) as u32;
        let y = (i / SCREEN_WIDTH) as u32;

        // Draw the pixel at (x, y) scaled up into a rectangle by SCALE
        let rect = Rect::new((x * SCALE) as i32, (y * SCALE) as i32, SCALE, SCALE);
        canvas.fill_rect(rect).unwrap();
    }

    canvas.present();
}

fn keycode_to_c8(key: Keycode) -> Option<usize> {
    match key {
        Keycode::Num1 => Some(0x1),
        Keycode::Num2 => Some(0x2),
        Keycode::Num3 => Some(0x3),
        Keycode::Num4 => Some(0xC),
        Keycode::Q => Some(0x4),
        Keycode::W => Some(0x5),
        Keycode::E => Some(0x6),
        Keycode::R => Some(0xD),
        Keycode::A => Some(0x7),
        Keycode::S => Some(0x8),
        Keycode::D => Some(0x9),
        Keycode::F => Some(0xE),
        Keycode::Z => Some(0xA),
        Keycode::X => Some(0x0),
        Keycode::C => Some(0xB),
        Keycode::V => Some(0xF),
        _ => None,
    }
}
