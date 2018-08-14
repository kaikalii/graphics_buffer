extern crate graphics;
extern crate image;

use std::ops;

use graphics::{draw_state::DrawState, types::Color, Graphics, ImageSize};
use image::{DynamicImage, Rgba, RgbaImage};

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

fn sign(p1: [f32; 2], p2: [f32; 2], p3: [f32; 2]) -> f32 {
    (p1[0] - p3[0]) * (p2[1] - p3[1]) - (p2[0] - p3[0]) * (p1[1] - p3[1])
}

fn triangle_contains(tri: &[[f32; 2]], point: [f32; 2]) -> bool {
    let b1 = sign(point, tri[0], tri[1]) < 0.0;
    let b2 = sign(point, tri[1], tri[2]) < 0.0;
    let b3 = sign(point, tri[2], tri[0]) < 0.0;
    b1 == b2 && b2 == b3
}

fn dist(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powf(2.0) + (a[1] - b[1]).powf(2.0)).powf(0.5)
}

fn map_to_triangle(point: [f32; 2], from_tri: &[[f32; 2]], to_tri: &[[f32; 2]]) -> [f32; 2] {
    let dists = [
        dist(from_tri[0], point),
        dist(from_tri[1], point),
        dist(from_tri[2], point),
    ];
    let dist_sum = dists[0] + dists[1] + dists[2];
    let sp = [
        [to_tri[0][0] * dists[0], to_tri[0][1] * dists[0]],
        [to_tri[1][0] * dists[0], to_tri[1][1] * dists[0]],
        [to_tri[2][0] * dists[0], to_tri[2][1] * dists[0]],
    ];
    [
        (sp[0][0] + sp[1][0] + sp[2][0]) / dist_sum,
        (sp[0][1] + sp[1][1] + sp[2][1]) / dist_sum,
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

#[derive(Debug, Clone)]
pub struct RenderBuffer {
    inner: RgbaImage,
}

impl RenderBuffer {
    pub fn new(width: u32, height: u32) -> RenderBuffer {
        RenderBuffer {
            inner: RgbaImage::new(width, height),
        }
    }
    pub fn clear(&mut self, color: [f32; 4]) {
        self.clear_color(color);
    }
    pub fn pixel(&self, x: u32, y: u32) -> [f32; 4] {
        color_rgba_f32(self.inner.get_pixel(x, y))
    }
    pub fn print(&self) {
        for x in 0..self.width() {
            if x % 2 == 1 {
                continue;
            }
            for y in 0..self.height() {
                print!("{}", if self.pixel(x, y)[0] == 0.0 { "." } else { "X" });
            }
            println!()
        }
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
        _color: &[f32; 4],
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
                for x in tl[0]..br[0] {
                    for y in tl[1]..br[1] {
                        if triangle_contains(tri, [x as f32, y as f32]) {
                            let scaled_tex_tri = tri_image_scale(tex_tri, texture.get_size());
                            let mapped_point =
                                map_to_triangle([x as f32, y as f32], tri, &scaled_tex_tri);
                            let texel = texture.get_pixel(
                                (mapped_point[0] as u32).min(texture.get_width() - 1),
                                (mapped_point[1] as u32).min(texture.get_height() - 1),
                            );
                            self.inner.put_pixel(x, y, *texel);
                        }
                    }
                }
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use graphics::{ellipse, image as draw_image};
    use image;
    #[test]
    fn circle_test() {
        let mut rb = RenderBuffer::new(100, 100);
        rb.clear([0.0, 0.0, 0.0, 1.0]);
        ellipse(
            [1.0, 0.0, 0.0, 1.0],
            [0.0, 0.0, 50.0, 50.0],
            [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            &mut rb,
        );
        rb.save("circle.png").unwrap();
    }
    #[test]
    fn image_test() {
        let texture = RenderBuffer::from(image::open("squares.png").unwrap());
        let mut rb = RenderBuffer::new(100, 100);
        rb.clear([0.0, 0.0, 0.0, 1.0]);
        draw_image(&texture, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]], &mut rb);
        rb.save("red_matt.png").unwrap();
    }
}
