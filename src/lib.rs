extern crate graphics;
extern crate image;

use std::{ops, path::Path};

use graphics::{draw_state::DrawState, math::Matrix2d, types::Color, Graphics, ImageSize};
use image::{DynamicImage, ImageError, Rgba, RgbaImage};

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
    let bary_a = ((t[1][1] - t[2][1]) * (p[0] - t[2][0]) + (t[2][0] - t[1][0]) * (p[1] - t[2][1]))
        / ((t[1][1] - t[2][1]) * (t[0][0] - t[2][0]) + (t[2][0] - t[1][0]) * (t[0][1] - t[2][1]));
    let bary_b = ((t[2][1] - t[0][1]) * (p[0] - t[2][0]) + (t[0][0] - t[2][0]) * (p[1] - t[2][1]))
        / ((t[1][1] - t[2][1]) * (t[0][0] - t[2][0]) + (t[2][0] - t[1][0]) * (t[0][1] - t[2][1]));
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

/// Returns the identity matrix: `[[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]`.
pub fn identity() -> Matrix2d {
    [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]]
}

/// A buffer that can be rendered to with Piston's graphics library.
#[derive(Debug, Clone)]
pub struct RenderBuffer {
    inner: RgbaImage,
}

impl RenderBuffer {
    /// Create a new `RenderBuffer` with the given witdth or height.
    pub fn new(width: u32, height: u32) -> RenderBuffer {
        RenderBuffer {
            inner: RgbaImage::new(width, height),
        }
    }
    /// Creates a new `RenderBuffer` by opening it from a file.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<RenderBuffer, ImageError> {
        image::open(path).map(|di| RenderBuffer::from(di))
    }
    /// Clear the buffer with a color.
    pub fn clear(&mut self, color: [f32; 4]) {
        self.clear_color(color);
    }
    /// Returns the color of the pixel at the given coordinates
    pub fn pixel(&self, x: u32, y: u32) -> [f32; 4] {
        color_rgba_f32(self.inner.get_pixel(x, y))
    }
    /// Sets the color of the pixel at the given coordinates
    pub fn set_pixel(&mut self, x: u32, y: u32, color: [f32; 4]) {
        self.inner.put_pixel(x, y, color_f32_rgba(&color));
    }
}

impl From<RgbaImage> for RenderBuffer {
    fn from(image: RgbaImage) -> Self {
        RenderBuffer { inner: image }
    }
}

impl From<DynamicImage> for RenderBuffer {
    fn from(image: DynamicImage) -> Self {
        RenderBuffer {
            inner: image.to_rgba(),
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
        // Convert color
        let color = color_f32_rgba(color);
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
                let tl = [tl[0].floor() as u32, tl[1].floor() as u32];
                let br = [br[0].ceil() as u32, br[1].ceil() as u32];
                // Render
                for x in tl[0]..br[0] {
                    for y in tl[1]..br[1] {
                        if triangle_contains(tri, [x as f32, y as f32]) {
                            self.inner.put_pixel(x, y, color);
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
                let tl = [tl[0].floor() as u32, tl[1].floor() as u32];
                let br = [br[0].ceil() as u32, br[1].ceil() as u32];
                // Render
                let scaled_tex_tri = tri_image_scale(tex_tri, texture.get_size());
                for x in tl[0]..br[0] {
                    for y in tl[1]..br[1] {
                        if triangle_contains(tri, [x as f32, y as f32]) {
                            let mapped_point =
                                map_to_triangle([x as f32, y as f32], tri, &scaled_tex_tri);
                            let texel = texture.get_pixel(
                                mapped_point[0].round() as u32,
                                mapped_point[1].round() as u32,
                            );
                            self.inner.put_pixel(
                                x,
                                y,
                                color_f32_rgba(&color_mul(&color_rgba_f32(texel), color)),
                            );
                        }
                    }
                }
            }
        });
    }
}
