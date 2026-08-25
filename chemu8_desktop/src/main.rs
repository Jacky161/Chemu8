#![windows_subsystem = "windows"]

mod audio;
mod menu;

use std::fs;

use crate::audio::SineWave;
use crate::menu::{create_text_texture, get_centered_rect};
use chemu8_core::Chip8;

use clap::Parser;

use sdl2::Sdl;
use sdl2::audio::{AudioDevice, AudioSpecDesired};
use sdl2::event::Event;
use sdl2::gfx::framerate::FPSManager;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::rect::Rect;
use sdl2::render::{Canvas, Texture, TextureCreator};
use sdl2::ttf::Sdl2TtfContext;
use sdl2::video::{Window, WindowContext};

use rfd::FileDialog;

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
    #[arg(short, long)]
    rom: Option<String>,

    /// Enable all Cowgod quirks. Some games rely on an inaccurate implementation of
    /// certain instructions and won't run properly without this being enabled.
    #[arg(short, long, default_value_t = true)]
    quirks: bool,
}

enum AppState<'a> {
    MENU,
    OPTIONS,
    ERROR(Texture<'a>),
    PLAYING,
}

fn main() {
    let args = Args::parse();

    let mut chip8 = Chip8::new(args.quirks, args.quirks, args.quirks, args.quirks);
    if let Some(rom_path) = args.rom {
        let rom_data = fs::read(rom_path).expect("Failed to read ROM!");
        chip8.load(&rom_data);
    }

    // Setup SDL and Display
    let sdl_context = sdl2::init().unwrap();
    let ttf_context = sdl2::ttf::init().unwrap();
    let audio_device = setup_audio(&sdl_context);
    let (fps_manager, canvas) = setup_display(&sdl_context);

    run_loop(
        chip8,
        &sdl_context,
        &ttf_context,
        canvas,
        fps_manager,
        audio_device,
    );
}

fn read_ch8_rom_from_picker(chip8: &mut Chip8) -> Result<(), String> {
    let file = FileDialog::new()
        .add_filter("Chip8 ROM", &["ch8"])
        .set_directory("./")
        .set_title("Select a ROM to Play!")
        .pick_file();

    match file {
        Some(path) => {
            let rom_data = fs::read(path);

            if let Ok(rom_bytes) = rom_data {
                chip8.load(&rom_bytes);
                return Ok(());
            }
        }
        None => {
            return Err(String::from("No ROM provided."));
        }
    }

    Err(String::from("Failed to read provided rom."))
}

fn setup_display(sdl_context: &Sdl) -> (FPSManager, Canvas<Window>) {
    // SETUP DISPLAY
    let video_subsystem = sdl_context.video().unwrap();
    let window = video_subsystem
        .window("Chemu8", WINDOW_WIDTH, WINDOW_HEIGHT)
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
    ttf_context: &Sdl2TtfContext,
    mut canvas: Canvas<Window>,
    mut fps_manager: FPSManager,
    audio_device: AudioDevice<SineWave>,
) {
    let mut event_pump = sdl_context.event_pump().unwrap();
    let texture_creator = canvas.texture_creator();
    let menu_texture = create_text_texture(
        "Welcome to Chemu8!\n1. Play\n2. Settings\n3. Exit",
        64,
        &texture_creator,
        ttf_context,
    )
    .unwrap();
    let mut options_texture = get_options_texture(&chip8, &texture_creator, ttf_context).unwrap();
    let mut state = AppState::MENU;

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
                } => match state {
                    AppState::PLAYING => {
                        if let Some(c8key) = keycode_to_c8(key) {
                            chip8.set_key(c8key, true);
                        }
                    }
                    _ => {}
                },
                Event::KeyUp {
                    keycode: Some(key), ..
                } => match state {
                    AppState::PLAYING => {
                        if let Some(c8key) = keycode_to_c8(key) {
                            chip8.set_key(c8key, false);
                        }
                    }
                    AppState::MENU => match key {
                        Keycode::Num1 => {
                            // Load ROM
                            if !chip8.rom_loaded() {
                                match read_ch8_rom_from_picker(&mut chip8) {
                                    Ok(..) => {
                                        state = AppState::PLAYING;
                                    }
                                    Err(msg) => {
                                        state = AppState::ERROR(create_text_texture(&format!("Error: {}\n1. Back", msg), 64, &texture_creator, ttf_context).unwrap());
                                    }
                                }
                            } else {
                                state = AppState::PLAYING;
                            }
                        }
                        Keycode::Num2 => {
                            state = AppState::OPTIONS;
                        }
                        Keycode::Num3 => break 'gameloop,
                        _ => {}
                    }
                    AppState::OPTIONS => {
                        match key {
                            Keycode::Num1 => {
                                chip8.quirk_set(true);
                            }
                            Keycode::Num2 => {
                                chip8.quirk_set(false);
                            }
                            Keycode::Num3 => {
                                chip8.quirk_8xy6 = !chip8.quirk_8xy6;
                            }
                            Keycode::Num4 => {
                                chip8.quirk_8xye = !chip8.quirk_8xye;
                            }
                            Keycode::Num5 => {
                                chip8.quirk_fx55 = !chip8.quirk_fx55;
                            }
                            Keycode::Num6 => {
                                chip8.quirk_fx65 = !chip8.quirk_fx65;
                            }
                            Keycode::Num7 => {
                                state = AppState::MENU;
                            }
                            _ => {}
                        }
                        options_texture =
                            get_options_texture(&chip8, &texture_creator, ttf_context).unwrap();
                    }
                    AppState::ERROR(_) => {
                        match key {
                            Keycode::Num1 => {
                                state = AppState::MENU;
                            }
                            _ => {}
                        }
                    }
                },
                _ => {}
            }
        }

        match &state {
            AppState::MENU => {
                draw_centered_text(&menu_texture, &mut canvas);
            }
            AppState::OPTIONS => {
                draw_centered_text(&options_texture, &mut canvas);
            }
            AppState::ERROR(error_texture) => {
                draw_centered_text(error_texture, &mut canvas);
            }
            AppState::PLAYING => {
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
            }
        }

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

fn draw_centered_text(text_texture: &Texture, canvas: &mut Canvas<Window>) {
    let query = text_texture.query();

    // If the text is too big for the screen, downscale it (and center irregardless)
    let padding = 64;
    let target = get_centered_rect(
        query.width,
        query.height,
        (WINDOW_WIDTH - padding) as u32,
        (WINDOW_HEIGHT - padding) as u32,
    );

    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.copy(&text_texture, None, Some(target)).unwrap();
    canvas.present();
}

fn get_options_texture<'a>(
    chip8: &Chip8,
    texture_creator: &'a TextureCreator<WindowContext>,
    ttf_context: &Sdl2TtfContext,
) -> Result<Texture<'a>, String> {
    let options_text = format!(
        "Options\n1. Toggle Quirks ON\n2. Toggle Quirks OFF\n3. quirk_8xy6 {}\n4. quirk_8xye {}\n5. quirk_fx55 {}\n6. quirk_fx65 {}\n7. Back",
        get_bool_option_text(chip8.quirk_8xy6),
        get_bool_option_text(chip8.quirk_8xye),
        get_bool_option_text(chip8.quirk_fx55),
        get_bool_option_text(chip8.quirk_fx65),
    );

    create_text_texture(&options_text, 44, &texture_creator, ttf_context)
}

fn get_bool_option_text(state: bool) -> &'static str {
    if state { "ON" } else { "OFF" }
}
