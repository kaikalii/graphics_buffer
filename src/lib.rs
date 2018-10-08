//! This library provides a buffer which can be used as a render target for [Piston's graphics library](https://github.com/PistonDevelopers/graphics). This buffer can be loaded from and/or saved to a file on disk. This allows for things like screenshots in games.

// !There is also an optional feature for `RenderBuffer` that allows it to be converted into a `G2dTexture` so that it can be rendered with [`piston_window`](https://github.com/PistonDevelopers/piston_window). To enable this, add `features = ["piston_window_texture"]` to the `graphics_buffer` dependency in your `cargo.toml`.

extern crate bit_vec;
extern crate graphics;
extern crate image;
#[cfg(feature = "piston_window_texture")]
extern crate piston_window;
extern crate rusttype;
#[macro_use]
extern crate failure;

mod glyphs;
pub use glyphs::*;

use std::{fmt, ops, path::Path};

use bit_vec::BitVec;
use graphics::{draw_state::DrawState, math::Matrix2d, types::Color, Graphics, ImageSize};
use image::{DynamicImage, GenericImageView, ImageResult, Rgba, RgbaImage};
#[cfg(feature = "piston_window_texture")]
use piston_window::{
    texture::{CreateTexture, Format},
    G2dTexture, GfxFactory, TextureSettings,
};

/// Returns the identity matrix: `[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]`.
pub fn identity() -> Matrix2d {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
}

/// An Error type for `RenderBuffer`.
#[derive(Debug, Clone, Fail)]
pub enum Error {
    ContainerTooSmall(usize, usize),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::ContainerTooSmall(len, area) => write!(
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

/// A buffer that can be rendered to with Piston's graphics library.
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
    pub fn open<P: AsRef<Path>>(path: P) -> ImageResult<RenderBuffer> {
        image::open(path).map(|di| RenderBuffer::from(di))
    }
    /// Creates a new `RenderBuffer` by decoding image data.
    pub fn decode_from_bytes(bytes: &[u8]) -> ImageResult<RenderBuffer> {
        image::load_from_memory(bytes).map(|di| RenderBuffer::from(di))
    }
    /// Clear the buffer with a color.
    pub fn clear(&mut self, color: [f32; 4]) {
        self.clear_color(color);
    }
    /// Returns the color of the pixel at the given coordinates.
    pub fn pixel(&self, x: u32, y: u32) -> [f32; 4] {
        color_rgba_f32(self.inner.get_pixel(x, y))
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
        factory: &mut GfxFactory,
        settings: &TextureSettings,
    ) -> Result<G2dTexture, failure::Error> {
        Ok(G2dTexture::from_image(factory, &self.inner, settings)?)
    }
}

#[cfg(feature = "piston_window_texture")]
impl CreateTexture<()> for RenderBuffer {
    type Error = failure::Error;
    fn create<S: Into<[u32; 2]>>(
        _factory: &mut (),
        _format: Format,
        memory: &[u8],
        size: S,
        _settings: &TextureSettings,
    ) -> Result<Self, failure::Error> {
        let size = size.into();
        Ok(RenderBuffer::from(
            RgbaImage::from_raw(size[0], size[1], memory.to_vec()).ok_or(
                Error::ContainerTooSmall(memory.len(), (size[0] * size[1]) as usize),
            )?,
        ))
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
        F: FnMut(&mut FnMut(&[[f32; 2]])),
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
                let tl = [tl[0].floor() as i32, tl[1].floor() as i32];
                let br = [br[0].ceil() as i32, br[1].ceil() as i32];
                // Render
                for x in tl[0]..br[0] {
                    for y in tl[1]..br[1] {
                        if x >= 0
                            && x < self.width() as i32
                            && y >= 0
                            && y < self.height() as i32
                            && triangle_contains(tri, [x as f32, y as f32])
                            && !self.used[x as usize].get(y as usize).unwrap_or(true)
                        {
                            let under_color =
                                color_rgba_f32(self.inner.get_pixel(x as u32, y as u32));
                            let layered_color = layer_color(&color, &under_color);
                            self.inner.put_pixel(
                                x as u32,
                                y as u32,
                                color_f32_rgba(&layered_color),
                            );
                            self.used[x as usize].set(y as usize, true);
                        }
                    }
                }
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
        F: FnMut(&mut FnMut(&[[f32; 2]], &[[f32; 2]])),
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
                let tl = [tl[0].floor() as i32, tl[1].floor() as i32];
                let br = [br[0].ceil() as i32, br[1].ceil() as i32];
                // Render
                let scaled_tex_tri = tri_image_scale(tex_tri, texture.get_size());
                for x in tl[0]..br[0] {
                    for y in tl[1]..br[1] {
                        if x >= 0
                            && x < self.width() as i32
                            && y >= 0
                            && y < self.height() as i32
                            && triangle_contains(tri, [x as f32, y as f32])
                        {
                            let mapped_point =
                                map_to_triangle([x as f32, y as f32], tri, &scaled_tex_tri);
                            let texel = color_rgba_f32(texture.get_pixel(
                                (mapped_point[0].round() as u32).min(texture.width() - 1),
                                (mapped_point[1].round() as u32).min(texture.height() - 1),
                            ));
                            let over_color = color_mul(color, &texel);
                            let under_color = color_rgba_f32(self.get_pixel(x as u32, y as u32));
                            let layered_color = layer_color(&over_color, &under_color);
                            self.inner.put_pixel(
                                x as u32,
                                y as u32,
                                color_f32_rgba(&layered_color),
                            );
                            self.used[x as usize].set(y as usize, true);
                        }
                    }
                }
            }
        });
    }
}

fn color_f32_rgba(color: &[f32; 4]) -> Rgba<u8> {
    Rgba {
        data: [
            (color[0] * 255.0) as u8,
            (color[1] * 255.0) as u8,
            (color[2] * 255.0) as u8,
            (color[3] * 255.0) as u8,
        ],
    }
}

fn color_rgba_f32(color: &Rgba<u8>) -> [f32; 4] {
    [
        (color.data[0] as f32) / 255.0,
        (color.data[1] as f32) / 255.0,
        (color.data[2] as f32) / 255.0,
        (color.data[3] as f32) / 255.0,
    ]
}

fn color_mul(a: &[f32; 4], b: &[f32; 4]) -> [f32; 4] {
    [a[0] * b[0], a[1] * b[1], a[2] * b[2], a[3] * b[3]]
}

fn layer_color(over: &[f32; 4], under: &[f32; 4]) -> [f32; 4] {
    let over_weight = over[3];
    let under_weight = 1.0 - over_weight;
    [
        over_weight * over[0] + under_weight * under[0],
        over_weight * over[1] + under_weight * under[1],
        over_weight * over[2] + under_weight * under[2],
        over[3].max(under[3]),
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
