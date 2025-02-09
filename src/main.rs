use crate::app::App;
use fonts::Font;
use log::debug;
use window::Window;

mod app;
mod fonts;
mod window;

const FONT_SIZE: usize = 30;
const WINDOW_SIZE: (usize, usize) = (4, 2);

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

	let (mut window, mut event_queue) = Window::new(
		(WINDOW_SIZE.0 * FONT_SIZE / 2) as u32,
		// +2 for rendering the top and bottom borders (1px each)
		(WINDOW_SIZE.1 * FONT_SIZE) as u32 + 2,
		App::new(font),
	);

	loop {
		event_queue.blocking_dispatch(&mut window).unwrap();

		if !window.app.running() {
			debug!("exiting example");
			break;
		}
	}

	window.app.exit();
}
