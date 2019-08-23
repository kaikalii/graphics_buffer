#![deny(missing_docs)]

/*!
This library provides a buffer which can be used as a render target for
[Piston's graphics library](https://github.com/PistonDevelopers/graphics).
This buffer can be loaded from and/or saved to a file on disk. This allows
for things like screenshots in games.

There is also an optional feature for `RenderBuffer` that allows it to be
converted into a `G2dTexture` so that it can be rendered with
[`piston_window`](https://github.com/PistonDevelopers/piston_window). To
enable this, add `features = ["piston_window_texture"]` to the `graphics_buffer`
dependency in your `cargo.toml`. More about this feature can be found in
the [`RenderBuffer` documentation](struct.RenderBuffer.html).
*/

mod glyphs;
pub use crate::glyphs::*;

use std::{error, fmt, fs::File, ops, path::Path};

use bit_vec::BitVec;
use graphics::{draw_state::DrawState, math::Matrix2d, types::Color, Graphics, ImageSize};
use image::{DynamicImage, GenericImageView, ImageResult, Rgba, RgbaImage};
#[cfg(feature = "piston_window_texture")]
use piston_window::{G2dTexture, G2dTextureContext};
use png::{Decoder as PngDecoder, Limits};
use rayon::prelude::*;
use texture::{CreateTexture, Format, TextureSettings, UpdateTexture};

/// The identity matrix: `[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]`.
pub const IDENTITY: Matrix2d = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];

/// An Error type for `RenderBuffer`.
#[derive(Debug, Clone)]
pub enum Error {
    /// Pixels/bytes mismatch when creating texture
    SizeMismatch(usize, usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::SizeMismatch(len, area) => write!(
                f,
                "Container is too small for the given dimensions. \
                 \nContainer has {} bytes, which encode {} pixels, \
                 \nbut the given demensions contain {} pixels",
                len,
                len / 4,
                area
            ),
        }
    }
}

impl error::Error for Error {}

/**
A buffer that can be rendered to with Piston's graphics library.

Enabling the `piston_window_texture` feature exposes a function called
`to_g2d_texture`, with the following signature:
```ignore
pub fn to_g2d_texture(
    &self,
    context: &mut G2dTextureContext,
    settings: &TextureSettings,
) -> Result<G2dTexture, Box<dyn std::error::Error>>;
```
*/
#[derive(Debug, Clone)]
pub struct RenderBuffer {
    inner: RgbaImage,
    used: Vec<BitVec>,
}

impl RenderBuffer {
    /// Create a new `RenderBuffer` with the given witdth or height.
    pub fn new(width: u32, height: u32) -> RenderBuffer {
        RenderBuffer {
            inner: RgbaImage::new(width, height),
            used: vec![BitVec::from_elem(height as usize, false); width as usize],
        }
    }
    /// Creates a new `RenderBuffer` by opening it from a file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<RenderBuffer, Box<dyn error::Error>> {
        if path
            .as_ref()
            .extension()
            .map(|ext| ext == "png")
            .unwrap_or(false)
        {
            let (info, mut reader) = PngDecoder::new_with_limits(
                File::open(&path)?,
                Limits {
                    bytes: std::usize::MAX,
                },
            )
            .read_info()?;
            let mut buf = vec![0; info.buffer_size()];
            reader.next_frame(&mut buf)?;
            Ok(
                if let Some(image) = image::RgbaImage::from_raw(info.width, info.height, buf) {
                    image.into()
                } else {
                    image::open(path)?.into()
                },
            )
        } else {
            Ok(image::open(path)?.into())
        }
    }
    /// Creates a new `RenderBuffer` by decoding image data.
    pub fn decode_from_bytes(bytes: &[u8]) -> ImageResult<RenderBuffer> {
        image::load_from_memory(bytes).map(RenderBuffer::from)
    }
    /// Clear the buffer with a color.
    pub fn clear(&mut self, color: [f32; 4]) {
        self.clear_color(color);
    }
    /// Returns the color of the pixel at the given coordinates.
    pub fn pixel(&self, x: u32, y: u32) -> [f32; 4] {
        color_rgba_f32(*self.inner.get_pixel(x, y))
    }
    /// Sets the color of the pixel at the given coordinates.
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [f32; 4]) {
        self.inner.put_pixel(x, y, color_f32_rgba(&color));
    }
    fn reset_used(&mut self) {
        let (width, height) = self.inner.dimensions();
        self.used = vec![BitVec::from_elem(height as usize, false); width as usize];
    }
    /// Creates a `G2dTexture` from the `RenderBuffer` for drawing to a `PistonWindow`.
    #[cfg(feature = "piston_window_texture")]
    pub fn to_g2d_texture(
        &self,
        context: &mut G2dTextureContext,
        settings: &TextureSettings,
    ) -> Result<G2dTexture, Box<dyn error::Error>> {
        Ok(G2dTexture::from_image(context, &self.inner, settings)?)
    }
}

impl CreateTexture<()> for RenderBuffer {
    type Error = Error;
    fn create<S: Into<[u32; 2]>>(
        _factory: &mut (),
        _format: Format,
        memory: &[u8],
        size: S,
        _settings: &TextureSettings,
    ) -> Result<Self, Error> {
        let size = size.into();
        Ok(RenderBuffer::from(
            RgbaImage::from_raw(size[0], size[1], memory.to_vec()).ok_or(Error::SizeMismatch(
                memory.len(),
                (size[0] * size[1]) as usize,
            ))?,
        ))
    }
}

impl UpdateTexture<()> for RenderBuffer {
    type Error = Error;
    fn update<O, S>(
        &mut self,
        _factory: &mut (),
        _format: Format,
        memory: &[u8],
        offset: O,
        size: S,
    ) -> Result<(), Self::Error>
    where
        O: Into<[u32; 2]>,
        S: Into<[u32; 2]>,
    {
        let offset = offset.into();
        let size = size.into();
        let new_image = RenderBuffer::from(
            RgbaImage::from_raw(size[0], size[1], memory.to_vec()).ok_or(Error::SizeMismatch(
                memory.len(),
                (size[0] * size[1]) as usize,
            ))?,
        );
        for i in 0..size[0] {
            for j in 0..size[1] {
                let color = new_image.pixel(i, j);
                self.set_pixel(i + offset[0], j + offset[1], color);
            }
        }
        Ok(())
    }
}

impl From<RgbaImage> for RenderBuffer {
    fn from(image: RgbaImage) -> Self {
        let (width, height) = image.dimensions();
        RenderBuffer {
            inner: image,
            used: vec![BitVec::from_elem(height as usize, false); width as usize],
        }
    }
}

impl From<DynamicImage> for RenderBuffer {
    fn from(image: DynamicImage) -> Self {
        let (width, height) = image.dimensions();
        RenderBuffer {
            inner: image.to_rgba(),
            used: vec![BitVec::from_elem(height as usize, false); width as usize],
        }
    }
}

impl ops::Deref for RenderBuffer {
    type Target = RgbaImage;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl ImageSize for RenderBuffer {
    fn get_size(&self) -> (u32, u32) {
        self.inner.dimensions()
    }
}

impl Graphics for RenderBuffer {
    type Texture = RenderBuffer;
    fn clear_color(&mut self, color: Color) {
        for (_, _, pixel) in self.inner.enumerate_pixels_mut() {
            *pixel = color_f32_rgba(&color);
        }
    }
    fn clear_stencil(&mut self, _value: u8) {}
    fn tri_list<F>(&mut self, _draw_state: &DrawState, color: &[f32; 4], mut f: F)
    where
        F: FnMut(&mut dyn FnMut(&[[f32; 2]])),
    {
        self.reset_used();
        // Render Triangles
        f(&mut |vertices| {
            for tri in vertices.chunks(3) {
                // Get tri bounds for efficiency
                let mut tl = [0f32, 0f32];
                let mut br = [0f32, 0f32];
                for v in tri {
                    tl[0] = tl[0].min(v[0]);
                    tl[1] = tl[1].min(v[1]);
                    br[0] = br[0].max(v[0]);
                    br[1] = br[1].max(v[1]);
                }
                let tl = [tl[0].floor().max(0.0) as i32, tl[1].floor().max(0.0) as i32];
                let br = [
                    br[0].ceil().min(self.width() as f32) as i32,
                    br[1].ceil().min(self.height() as f32) as i32,
                ];
                // Render
                let inner = &self.inner;
                let used = &self.used;
                (tl[0]..br[0]).into_par_iter().for_each(|x| {
                    let mut entered = false;
                    for y in tl[1]..br[1] {
                        if triangle_contains(tri, [x as f32, y as f32]) {
                            entered = true;
                            if !used[x as usize].get(y as usize).unwrap_or(true) {
                                let under_color =
                                    color_rgba_f32(*inner.get_pixel(x as u32, y as u32));
                                let layered_color = layer_color(&color, &under_color);
                                unsafe {
                                    (inner as *const RgbaImage as *mut RgbaImage)
                                        .as_mut()
                                        .unwrap()
                                        .put_pixel(
                                            x as u32,
                                            y as u32,
                                            color_f32_rgba(&layered_color),
                                        );
                                    (used as *const Vec<BitVec> as *mut Vec<BitVec>)
                                        .as_mut()
                                        .unwrap()[x as usize]
                                        .set(y as usize, true);
                                }
                            }
                        } else if entered {
                            break;
                        }
                    }
                });
            }
        });
    }
    fn tri_list_uv<F>(
        &mut self,
        _draw_state: &DrawState,
        color: &[f32; 4],
        texture: &Self::Texture,
        mut f: F,
    ) where
        F: FnMut(&mut dyn FnMut(&[[f32; 2]], &[[f32; 2]])),
    {
        self.reset_used();
        // Render Triangles
        f(&mut |vertices, tex_vertices| {
            for (tri, tex_tri) in vertices.chunks(3).zip(tex_vertices.chunks(3)) {
                // Get tri bounds for efficiency
                let mut tl = [0f32, 0f32];
                let mut br = [0f32, 0f32];
                for v in tri {
                    tl[0] = tl[0].min(v[0]);
                    tl[1] = tl[1].min(v[1]);
                    br[0] = br[0].max(v[0]);
                    br[1] = br[1].max(v[1]);
                }
                let tl = [tl[0].floor().max(0.0) as i32, tl[1].floor().max(0.0) as i32];
                let br = [
                    br[0].ceil().min((self.width() - 1) as f32) as i32,
                    br[1].ceil().min((self.height() - 1) as f32) as i32,
                ];
                let avg_y = ((tri[0][1] + tri[1][1] + tri[2][1]) / 3.0) as i32;
                let vert_center = (br[1] - tl[1]) / 2;
                let vertical_balance_top = avg_y < vert_center;
                // Render
                let scaled_tex_tri = tri_image_scale(tex_tri, texture.get_size());
                let inner = &self.inner;
                let used = &self.used;
                (tl[0]..br[0]).into_par_iter().for_each(|x| {
                    let mut entered = false;
                    let range: Box<dyn Iterator<Item = i32>> = if vertical_balance_top {
                        Box::new(tl[1]..br[1])
                    } else {
                        Box::new((tl[1]..br[1]).rev())
                    };
                    for y in range {
                        if triangle_contains(tri, [x as f32, y as f32]) {
                            entered = true;
                            let mapped_point =
                                map_to_triangle([x as f32, y as f32], tri, &scaled_tex_tri);
                            let texel = color_rgba_f32(*texture.get_pixel(
                                (mapped_point[0].round() as u32).min(texture.width() - 1),
                                (mapped_point[1].round() as u32).min(texture.height() - 1),
                            ));
                            let over_color = color_mul(color, &texel);
                            let under_color = color_rgba_f32(*inner.get_pixel(x as u32, y as u32));
                            let layered_color = layer_color(&over_color, &under_color);
                            unsafe {
                                (inner as *const RgbaImage as *mut RgbaImage)
                                    .as_mut()
                                    .unwrap()
                                    .put_pixel(x as u32, y as u32, color_f32_rgba(&layered_color));
                                (used as *const Vec<BitVec> as *mut Vec<BitVec>)
                                    .as_mut()
                                    .unwrap()[x as usize]
                                    .set(y as usize, true);
                            }
                        } else if entered {
                            break;
                        }
                    }
                });
            }
        });
    }
}

fn color_f32_rgba(color: &[f32; 4]) -> Rgba<u8> {
    Rgba([
        (color[0] * 255.0) as u8,
        (color[1] * 255.0) as u8,
        (color[2] * 255.0) as u8,
        (color[3] * 255.0) as u8,
    ])
}

fn color_rgba_f32(color: Rgba<u8>) -> [f32; 4] {
    [
        f32::from(color[0]) / 255.0,
        f32::from(color[1]) / 255.0,
        f32::from(color[2]) / 255.0,
        f32::from(color[3]) / 255.0,
    ]
}

fn color_mul(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

fn layer_color(over: &[f32; 4], under: &[f32; 4]) -> [f32; 4] {
    let over_weight = 1.0 - (1.0 - over[3]).powf(2.0);
    let under_weight = 1.0 - over_weight;
    [
        over_weight * over[0] + under_weight * under[0],
        over_weight * over[1] + under_weight * under[1],
        over_weight * over[2] + under_weight * under[2],
        (over[3].powf(2.0) + under[3].powf(2.0)).sqrt().min(1.0),
    ]
}

fn sign(p1: [f32; 2], p2: [f32; 2], p3: [f32; 2]) -> f32 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}

fn triangle_contains(tri: &[[f32; 2]], point: [f32; 2]) -> bool {
    let b1 = sign(point, tri[0], tri[1]) < 0.0;
    let b2 = sign(point, tri[1], tri[2]) < 0.0;
    let b3 = sign(point, tri[2], tri[0]) < 0.0;
    b1 == b2 && b2 == b3
}

fn map_to_triangle(point: [f32; 2], from_tri: &[[f32; 2]], to_tri: &[[f32; 2]]) -> [f32; 2] {
    let t = from_tri;
    let p = point;
    // Computer some values that are used multiple times
    let a = t[1][1] - t[2][1];
    let b = p[0] - t[2][0];
    let c = t[2][0] - t[1][0];
    let d = p[1] - t[2][1];
    let e = t[0][0] - t[2][0];
    let f = t[0][1] - t[2][1];
    let g = t[2][1] - t[0][1];
    let ae_cf = a * e + c * f;
    let bary_a = (a * b + c * d) / ae_cf;
    let bary_b = (g * b + e * d) / ae_cf;
    let bary_c = 1.0 - bary_a - bary_b;
    [
        bary_a * to_tri[0][0] + bary_b * to_tri[1][0] + bary_c * to_tri[2][0],
        bary_a * to_tri[0][1] + bary_b * to_tri[1][1] + bary_c * to_tri[2][1],
    ]
}

fn point_image_scale(point: [f32; 2], size: (u32, u32)) -> [f32; 2] {
    [point[0] * size.0 as f32, point[1] * size.1 as f32]
}

fn tri_image_scale(tri: &[[f32; 2]], size: (u32, u32)) -> [[f32; 2]; 3] {
    [
        point_image_scale(tri[0], size),
        point_image_scale(tri[1], size),
        point_image_scale(tri[2], size),
    ]
}
