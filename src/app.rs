use crate::fonts::Font;
use log::{debug, log_enabled, Level};

/// Responsible for event handling and drawing to screen
pub struct App {
	// Some internal state
	font: Font,
}

impl App {
	pub fn new(font: Font) -> Self {
		App { font }
	}
	pub fn draw(&mut self, canvas: &mut [u8], width: u32, height: u32) {
		let line_count = height as usize / self.font.height;
		// 24-bit colors in ARGB format
		const BACKGROUND: u32 = 0xff000000;
		canvas.chunks_exact_mut(4).for_each(|chunk| {
			let array: &mut [u8; 4] = chunk.try_into().unwrap();
			*array = BACKGROUND.to_le_bytes();
		});
		let mut draw_line = |index, text: &str, selection: bool| {
			let top_line = index * width as usize * self.font.height * 4;
			for (i, symbol) in text.char_indices() {
				let glyph = match self.font.get_glyph(symbol) {
					Some(x) => x,
					None => continue,
				};
				let top_left = top_line + i * self.font.width * 4;
				for j in 0..self.font.height {
					for i in 0..self.font.width {
						let index = top_left + 4 * (i + j * width as usize);
						let mut pixel_value = glyph[i + j * self.font.width];
						if selection {
							pixel_value = 0xff - pixel_value;
						}
						canvas[index] = pixel_value;
						canvas[index + 1] = pixel_value;
						canvas[index + 2] = pixel_value;
					}
				}
			}
		};
		draw_line(0, "Demo", false);
		// TODO: Handle text and cursor rendering when the text width is greater than canvas width
	}
	pub fn exit(&self) {
		std::process::exit(0);
	}
}
