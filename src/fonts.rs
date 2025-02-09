use anyhow::{anyhow, Error, Result};
use resize::{px::Gray, Pixel::Gray8, Type::Triangle};

pub struct Font {
    pub width: usize,
    pub height: usize,
    pub glyphs: [[Vec<u8>; 32]; 3],
}

impl Font {
    /// Parses a pbm image file as the font atlas.
    /// The atlas should be `32` glyphs wide and `3` glyphs tall, starting at
    /// Space (' ') in standard ASCII ordering
    pub fn from_pbm(bytes: &[u8], font_size: usize) -> Result<Self> {
        let (image_width, image_height, pixel_data) = Self::parse_pbm(bytes)?;
        if image_width % 32 != 0 {
            return Err(anyhow!("Font atlas width is invalid: {}", image_width));
        }
        if image_height % 3 != 0 {
            return Err(anyhow!("Font atlas height is invalid: {}", image_height));
        }
        let glyph_original_size = (image_width / 32, image_height / 3);
        // TODO: Pre allocate glyph bitmaps here
        let mut glyphs_original: [[Vec<Gray<u8>>; 32]; 3] = Default::default();
        for (j, row) in glyphs_original.iter_mut().enumerate() {
            for (i, glyph) in row.iter_mut().enumerate() {
                let top_left = i as usize * glyph_original_size.0
                    + j as usize * glyph_original_size.1 * image_width;
                for j in 0..glyph_original_size.1 {
                    for i in 0..glyph_original_size.0 {
                        let pixel_value = pixel_data[top_left + i + j * image_width];
                        glyph.push(Gray::new(pixel_value));
                    }
                }
            }
        }
        let mut glyphs_resized: [[Vec<Gray<u8>>; 32]; 3] = Default::default();
        let glyph_new_size = (font_size / 2, font_size);
        let mut resizer = resize::new(
            glyph_original_size.0,
            glyph_original_size.1,
            glyph_new_size.0,
            glyph_new_size.1,
            Gray8,
            Triangle,
        )?;
        let mut glyphs: [[Vec<u8>; 32]; 3] = Default::default();
        for (j, row) in glyphs_original.iter().enumerate() {
            for (i, glyph) in row.iter().enumerate() {
                glyphs_resized[j][i] = vec![Gray::new(0); glyph_new_size.0 * glyph_new_size.1];
                resizer.resize(glyph, &mut glyphs_resized[j][i])?;
                glyphs[j][i] = glyphs_resized[j][i].iter().map(|i| i.0).collect();
            }
        }
        Ok(Self {
            width: glyph_new_size.0,
            height: glyph_new_size.1,
            glyphs,
        })
    }

    pub fn get_glyph(&self, symbol: char) -> Option<&[u8]> {
        if !(' '..='~').contains(&symbol) {
            return None;
        }
        let symbol_ascii = symbol as usize;
        Some(&self.glyphs[(symbol_ascii >> 5) - 1][symbol_ascii & 0x1f])
    }

    fn parse_pbm(bytes: &[u8]) -> Result<(usize, usize, Vec<u8>)> {
        let mut i = 0;
        let mut bitmap_offset = 0;
        let mut result = (
            Err(Error::msg("width")),
            Err(Error::msg("height")),
            Err(Error::msg("bitmap")),
        );
        for data in bytes.split(|i| *i == b'\n') {
            if i < 2 {
                // + 1 to account for the '\n' in between
                bitmap_offset += data.len() + 1;
            }
            if let Some(b'#') = data.first() {
                continue;
            }
            match i {
                0 => {
                    if data != b"P4" {
                        Err(anyhow!("Magic Word 'P4' not found"))?
                    }
                }
                1 => {
                    for (i, value) in data.split(|i| *i == b' ').enumerate() {
                        if i >= 2 {
                            Err(anyhow!("Failed to parse image width and height"))?;
                        }
                        let value: usize = std::str::from_utf8(value)?.parse()?;
                        if i == 0 {
                            result.0 = Ok(value);
                        }
                        if i == 1 {
                            result.1 = Ok(value);
                        }
                    }
                }
                _ => break,
            }
            i += 1;
        }
        let bitmap = &bytes[bitmap_offset..];
        let bitmap_decompressed = bitmap
            .iter()
            .flat_map(|i| {
                [
                    i & 0x80,
                    i & 0x40,
                    i & 0x20,
                    i & 0x10,
                    i & 8,
                    i & 4,
                    i & 2,
                    i & 1,
                ]
            })
            .map(|i| if i != 0 { 255 } else { 0 })
            .collect();
        // TODO: Handle the case when number of pixels isn't multiple of 8
        result.2 = Ok(bitmap_decompressed);
        Ok((result.0?, result.1?, result.2?))
    }
}
