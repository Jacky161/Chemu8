use sdl2::{
    pixels::Color,
    rect::Rect,
    render::{Texture, TextureCreator},
    rwops::RWops,
    ttf::Sdl2TtfContext,
    video::WindowContext,
};

use crate::{WINDOW_HEIGHT, WINDOW_WIDTH};

// Scale fonts to a reasonable size when they're too big (though they might look less smooth)
pub fn get_centered_rect(
    rect_width: u32,
    rect_height: u32,
    cons_width: u32,
    cons_height: u32,
) -> Rect {
    let wr = rect_width as f32 / cons_width as f32;
    let hr = rect_height as f32 / cons_height as f32;

    let (w, h) = if wr > 1f32 || hr > 1f32 {
        if wr > hr {
            println!("Scaling down! The text will look worse!");
            let h = (rect_height as f32 / wr) as i32;
            (cons_width as i32, h)
        } else {
            println!("Scaling down! The text will look worse!");
            let w = (rect_width as f32 / hr) as i32;
            (w, cons_height as i32)
        }
    } else {
        (rect_width as i32, rect_height as i32)
    };

    let cx = (WINDOW_WIDTH as i32 - w) / 2;
    let cy = (WINDOW_HEIGHT as i32 - h) / 2;
    Rect::new(cx, cy, w as u32, h as u32)
}

pub fn create_text_texture<'a>(
    text: &str,
    point_size: u16,
    texture_creator: &'a TextureCreator<WindowContext>,
    ttf_context: &Sdl2TtfContext,
) -> Result<Texture<'a>, String> {
    let font_bytes: &'static [u8] = include_bytes!("../assets/fonts/Roboto-Regular.ttf");
    let rwops = RWops::from_bytes(font_bytes)?;
    let font = ttf_context.load_font_from_rwops(rwops, point_size)?;

    let menu_surface = font
        .render(text)
        .blended_wrapped(Color::RGB(255, 255, 255), 0)
        .map_err(|e| e.to_string())?;

    texture_creator
        .create_texture_from_surface(&menu_surface)
        .map_err(|e| e.to_string())
}
