use fonts::Font;
use log::debug;

mod fonts;

const FONT_SIZE: usize = 30;

fn main() {
    env_logger::init();

    let font = Font::from_pbm(include_bytes!("./res/font_atlas.pbm"), FONT_SIZE).unwrap();
    let glyph = font.get_glyph('a').unwrap();
    let mut glyph_render = String::new();
    for i in 0..FONT_SIZE {
        for j in 0..FONT_SIZE / 2 {
            let char = glyph[i * FONT_SIZE / 2 + j];
            glyph_render.push_str(&format!("{:02x}", char));
        }
        glyph_render.push_str(&"\n");
    }
    debug!("{glyph_render}");
}
