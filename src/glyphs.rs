use std::{collections::HashMap, error};

use graphics::{
    character::{Character, CharacterCache},
    types::{FontSize, Scalar},
};
use rusttype::{point, Error, Font, GlyphId, Rect, Scale};

use super::*;

#[derive(Clone)]
struct CharacterDef {
    offset: [Scalar; 2],
    size: [Scalar; 2],
    texture: RenderBuffer,
}

impl CharacterDef {
    fn as_character<'a>(&'a self) -> Character<'a, RenderBuffer> {
        Character {
            offset: self.offset,
            size: self.size,
            texture: &self.texture,
        }
    }
}

/// A character cache for drawing text to a `RenderBuffer`.
#[derive(Clone)]
pub struct BufferGlyphs<'f> {
    characters: HashMap<(char, u32), CharacterDef>,
    font: Font<'f>,
}

impl<'f> BufferGlyphs<'f> {
    /// Loads a `BufferGlyphs` from an array of font data.
    pub fn from_bytes(bytes: &'f [u8]) -> Result<BufferGlyphs<'f>, Error> {
        Ok(BufferGlyphs {
            characters: HashMap::new(),
            font: Font::from_bytes(bytes)?,
        })
    }
    /// Loads a `BufferGlyphs` from a `Font`.
    pub fn from_font(font: Font<'f>) -> BufferGlyphs<'f> {
        BufferGlyphs {
            characters: HashMap::new(),
            font,
        }
    }
}

impl<'f> CharacterCache for BufferGlyphs<'f> {
    type Texture = RenderBuffer;
    type Error = Box<error::Error>;
    fn character<'a>(
        &'a mut self,
        font_size: FontSize,
        ch: char,
    ) -> Result<Character<'a, Self::Texture>, Self::Error> {
        let font = &self.font;
        Ok(self
            .characters
            .entry((ch, font_size))
            .or_insert_with(|| {
                let scale = Scale::uniform(font_size as f32);
                let glyph = font.glyph(ch).scaled(scale);
                let glyph = if glyph.id() == GlyphId(0) && glyph.shape().is_none() {
                    font.glyph('\u{FFFD}').scaled(scale)
                } else {
                    glyph
                };
                let h_metrics = glyph.h_metrics();
                let bounding_box = glyph.exact_bounding_box().unwrap_or(Rect {
                    min: point(0.0, 0.0),
                    max: point(0.0, 0.0),
                });
                let glyph = glyph.positioned(point(0.0, 0.0));
                let pixel_bounding_box = glyph.pixel_bounding_box().unwrap_or(Rect {
                    min: point(0, 0),
                    max: point(0, 0),
                });
                let pixel_bb_width = pixel_bounding_box.width() + 2;
                let pixel_bb_height = pixel_bounding_box.height() + 2;

                let mut texture = RenderBuffer::new(pixel_bb_width as u32, pixel_bb_height as u32);
                glyph.draw(|x, y, v| {
                    texture.set_pixel(x, y, [1.0, 1.0, 1.0, v]);
                });
                CharacterDef {
                    offset: [
                        bounding_box.min.x as Scalar - 1.0,
                        -pixel_bounding_box.min.y as Scalar + 1.0,
                    ],
                    size: [h_metrics.advance_width as Scalar, 0 as Scalar],
                    texture,
                }
            }).as_character())
    }
}
