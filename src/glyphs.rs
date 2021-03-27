use std::{io, path::Path};

use graphics::glyph_cache::rusttype;
use texture::TextureSettings;

use crate::RenderBuffer;

/// A character cache for drawing text to a `RenderBuffer`.
///
/// If the link to the `GlyphCache` type is not working,
/// try generating the docs yourself.
pub type BufferGlyphs<'a> = rusttype::GlyphCache<'a, (), RenderBuffer>;

/// Create a `BufferGlyphs` from some font data
#[allow(clippy::result_unit_err)]
pub fn buffer_glyphs_from_bytes(font_data: &[u8]) -> Result<BufferGlyphs, ()> {
    BufferGlyphs::from_bytes(font_data, (), TextureSettings::new())
}

/// Create a `BufferGlyphs` from a path to some font
pub fn buffer_glyphs_from_path<'a, P: AsRef<Path>>(font_path: P) -> io::Result<BufferGlyphs<'a>> {
    BufferGlyphs::new(font_path, (), TextureSettings::new())
}
