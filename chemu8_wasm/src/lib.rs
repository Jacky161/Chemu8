mod utils;

use chemu8_core::Chip8;
use js_sys::Uint8Array;
use wasm_bindgen::prelude::*;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement, KeyboardEvent};

const SCREEN_WIDTH: usize = 64;
const SCREEN_HEIGHT: usize = 32;

#[wasm_bindgen]
pub struct Chip8Wasm {
    chip8: Chip8,
    ctx: CanvasRenderingContext2d,
}

#[wasm_bindgen]
impl Chip8Wasm {
    #[wasm_bindgen(constructor)]
    pub fn new(canvas_id: &str) -> Result<Self, JsValue> {
        let chip8 = Chip8::new(true, true);

        let document = web_sys::window().unwrap().document().unwrap();
        let canvas = document.get_element_by_id(canvas_id).unwrap();
        let canvas: HtmlCanvasElement = canvas.dyn_into::<HtmlCanvasElement>()?;
        let ctx = canvas
            .get_context("2d")?
            .unwrap()
            .dyn_into::<CanvasRenderingContext2d>()?;

        Ok(Self { chip8, ctx })
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.chip8 = Chip8::new(true, true);
    }

    #[wasm_bindgen]
    pub fn tick(&mut self) {
        self.chip8.tick();
    }

    #[wasm_bindgen]
    pub fn tick_timers(&mut self) -> bool {
        self.chip8.tick_timers()
    }

    #[wasm_bindgen]
    pub fn notify_vblank(&mut self) {
        self.chip8.notify_vblank();
    }

    #[wasm_bindgen]
    pub fn keypress(&mut self, evt: KeyboardEvent, pressed: bool) {
        let key = evt.key();
        if let Some(k) = key2btn(&key) {
            self.chip8.set_key(k, pressed);
        }
    }

    #[wasm_bindgen]
    pub fn load_game(&mut self, data: Uint8Array) {
        self.chip8.load(&data.to_vec());
    }

    #[wasm_bindgen]
    pub fn draw_screen(&mut self, scale: usize) {
        for i in 0..(SCREEN_WIDTH * SCREEN_HEIGHT) {
            if self.chip8.screen[i] {
                let x = i % SCREEN_WIDTH;
                let y = i / SCREEN_WIDTH;
                self.ctx.fill_rect(
                    (x * scale) as f64,
                    (y * scale) as f64,
                    scale as f64,
                    scale as f64,
                );
            }
        }
    }
}

fn key2btn(key: &str) -> Option<usize> {
    match key {
        "1" => Some(0x1),
        "2" => Some(0x2),
        "3" => Some(0x3),
        "4" => Some(0xC),
        "q" | "Q" => Some(0x4),
        "w" | "W" => Some(0x5),
        "e" | "E" => Some(0x6),
        "r" | "R" => Some(0xD),
        "a" | "A" => Some(0x7),
        "s" | "S" => Some(0x8),
        "d" | "D" => Some(0x9),
        "f" | "F" => Some(0xE),
        "z" | "Z" => Some(0xA),
        "x" | "X" => Some(0x0),
        "c" | "C" => Some(0xB),
        "v" | "V" => Some(0xF),
        _ => None,
    }
}
