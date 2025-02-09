use crate::fonts::Font;

/// Responsible for event handling and drawing to screen
pub struct App {
	// Some internal state
	font: Font,
	text: String,
}

impl App {
	pub fn new(font: Font, text: String) -> Self {
		App { font, text }
	}
	pub fn draw(&mut self, canvas: &mut [u8], width: u32, _height: u32) {
		// 24-bit colors in ARGB format
		const BACKGROUND: u32 = 0xff000000;
		canvas.chunks_exact_mut(4).for_each(|chunk| {
			let array: &mut [u8; 4] = chunk.try_into().unwrap();
			*array = BACKGROUND.to_le_bytes();
		});
		for (i, symbol) in self.text.char_indices() {
			let glyph = match self.font.get_glyph(symbol) {
				Some(x) => x,
				None => continue,
			};
			let top_left = i * self.font.width * 4;
			for j in 0..self.font.height {
				for i in 0..self.font.width {
					let index = top_left + 4 * (i + j * width as usize);
					let pixel_value = glyph[i + j * self.font.width];
					canvas[index] = pixel_value;
					canvas[index + 1] = pixel_value;
					canvas[index + 2] = pixel_value;
				}
			}
		}
		// TODO: Handle text and cursor rendering when the text width is greater than canvas width
	}
	pub fn exit(&self) {
		std::process::exit(0);
	}
}
