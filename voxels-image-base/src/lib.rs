// This file is part of Hashlife3d.
//
// Hashlife3d is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Hashlife3d is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with Hashlife3d.  If not, see <https://www.gnu.org/licenses/>
extern crate voxels_math as math;
use math::Mappable;
use std::any;
use std::convert;
use std::error;
use std::fmt;
use std::io;
use std::io::prelude::*;

pub type Pixel = math::Vec4<u8>;

pub trait PixelBuffer:
    convert::AsMut<[Pixel]> + convert::AsRef<[Pixel]> + any::Any + Send + Sync
{
}

impl PixelBuffer for Vec<Pixel> {}

pub trait PixelBufferFactory {
    fn create_pixel_buffer(self, len: usize) -> io::Result<Box<PixelBuffer>>;
}

impl<T: FnOnce(usize) -> io::Result<Box<PixelBuffer>>> PixelBufferFactory for T {
    fn create_pixel_buffer(self, len: usize) -> io::Result<Box<PixelBuffer>> {
        self(len)
    }
}

pub struct DefaultPixelBufferFactory;

impl PixelBufferFactory for DefaultPixelBufferFactory {
    fn create_pixel_buffer(self, len: usize) -> io::Result<Box<PixelBuffer>> {
        let mut retval = Vec::new();
        retval.resize(len, Default::default());
        Ok(Box::new(retval))
    }
}

impl PixelBuffer for [Pixel] {}

pub struct Image {
    dimensions: math::Vec2<u32>,
    pixels: Box<PixelBuffer>,
}

impl Clone for Image {
    fn clone(&self) -> Self {
        Self {
            dimensions: self.dimensions,
            pixels: Box::new(self.pixels.as_ref().as_ref().to_owned()),
        }
    }
}

pub struct ImageSizeMismatchError {
    pub pixels: Box<PixelBuffer>,
}

impl error::Error for ImageSizeMismatchError {}

impl fmt::Debug for ImageSizeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct("ImageSizeMismatchError")
            .field("pixels", &())
            .finish()
    }
}

impl fmt::Display for ImageSizeMismatchError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str("image size mismatch error")
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum PPMMode {
    Binary,
    Text,
}

impl Default for PPMMode {
    fn default() -> Self {
        PPMMode::Binary
    }
}

pub struct ImageAsPPM<'a> {
    image: &'a Image,
    header_or_scanline: io::Cursor<Vec<u8>>,
    y: u32,
    mode: PPMMode,
}

impl<'a> ImageAsPPM<'a> {
    fn new(image: &'a Image, mode: PPMMode) -> Self {
        Self {
            image: image,
            header_or_scanline: io::Cursor::new(
                format!(
                    "{}\n{} {}\n255\n",
                    match mode {
                        PPMMode::Binary => "P6",
                        PPMMode::Text => "P3",
                    },
                    image.dimensions().x,
                    image.dimensions().y,
                ).into_bytes(),
            ),
            y: 0,
            mode: mode,
        }
    }
}

impl<'a> io::Read for ImageAsPPM<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        loop {
            match self.header_or_scanline.read(buf) {
                Ok(0) => {
                    if buf.len() != 0
                        && self.image.dimensions().x != 0
                        && self.y < self.image.dimensions().y
                    {
                        self.header_or_scanline.set_position(0);
                        match self.mode {
                            PPMMode::Binary => {
                                let scanline = self.header_or_scanline.get_mut();
                                scanline.resize(self.image.dimensions().x as usize * 3, 0);
                                for x in 0..self.image.dimensions().x {
                                    let pixel = self.image.get(math::Vec2::new(x, self.y));
                                    scanline[x as usize * 3] = pixel.x;
                                    scanline[x as usize * 3 + 1] = pixel.y;
                                    scanline[x as usize * 3 + 2] = pixel.z;
                                }
                            }
                            PPMMode::Text => {
                                self.header_or_scanline.get_mut().clear();
                                for x in 0..self.image.dimensions().x {
                                    let pixel = self.image.get(math::Vec2::new(x, self.y));
                                    write!(
                                        &mut self.header_or_scanline,
                                        "{} {} {} ",
                                        pixel.x, pixel.y, pixel.z,
                                    )?;
                                }
                                writeln!(&mut self.header_or_scanline)?;
                                self.header_or_scanline.set_position(0);
                            }
                        }
                        self.y = self.y + 1;
                        continue;
                    } else {
                        return Ok(0);
                    }
                }
                result => return result,
            }
        }
    }
}

fn get_line_offset_in_pixels(dimensions: math::Vec2<u32>, y: u32) -> usize {
    assert!(y < dimensions.y);
    get_line_offset_in_pixels_unchecked(dimensions, y)
}

fn get_pixel_offset_in_pixels(dimensions: math::Vec2<u32>, p: math::Vec2<u32>) -> usize {
    assert!(p.x < dimensions.x);
    assert!(p.y < dimensions.y);
    get_pixel_offset_in_pixels_unchecked(dimensions, p)
}

fn get_line_offset_in_pixels_unchecked(dimensions: math::Vec2<u32>, y: u32) -> usize {
    dimensions.x as usize * y as usize
}

fn get_pixel_offset_in_pixels_unchecked(dimensions: math::Vec2<u32>, p: math::Vec2<u32>) -> usize {
    p.x as usize + get_line_offset_in_pixels_unchecked(dimensions, p.y)
}

fn get_pixel_count(dimensions: math::Vec2<u32>) -> usize {
    (dimensions.x as usize)
        .checked_mul(dimensions.y as usize)
        .unwrap()
}

impl Image {
    pub fn dimensions(&self) -> math::Vec2<u32> {
        self.dimensions
    }
    pub fn get_line_offset_in_pixels(&self, y: u32) -> usize {
        get_line_offset_in_pixels(self.dimensions, y)
    }
    pub fn get_pixel_offset_in_pixels(&self, p: math::Vec2<u32>) -> usize {
        get_pixel_offset_in_pixels(self.dimensions, p)
    }
    pub fn get_line_offset_in_pixels_unchecked(&self, y: u32) -> usize {
        get_line_offset_in_pixels_unchecked(self.dimensions, y)
    }
    pub fn get_pixel_offset_in_pixels_unchecked(&self, p: math::Vec2<u32>) -> usize {
        get_pixel_offset_in_pixels_unchecked(self.dimensions, p)
    }
    pub fn get_pixel_count(&self) -> usize {
        get_pixel_count(self.dimensions)
    }
    pub fn clear(&mut self, clear_color: Pixel) {
        let pixels = &mut self.pixels.as_mut().as_mut()[0..get_pixel_count(self.dimensions)];
        for pixel in pixels {
            *pixel = clear_color;
        }
    }
    pub fn new(dimensions: math::Vec2<u32>, fill_color: Pixel) -> Self {
        let mut pixels = Vec::new();
        pixels.resize(get_pixel_count(dimensions), fill_color);
        Self {
            dimensions: dimensions,
            pixels: Box::new(pixels),
        }
    }
    pub fn with_pixel_buffer_factory<PBF: PixelBufferFactory>(
        dimensions: math::Vec2<u32>,
        fill_color: Pixel,
        pixel_buffer_factory: PBF,
    ) -> io::Result<Self> {
        let pixels = pixel_buffer_factory.create_pixel_buffer(get_pixel_count(dimensions))?;
        let mut retval = Self {
            dimensions: dimensions,
            pixels: pixels,
        };
        retval.clear(fill_color);
        Ok(retval)
    }
    pub fn from_pixels(
        dimensions: math::Vec2<u32>,
        pixels: Box<PixelBuffer>,
    ) -> Result<Self, ImageSizeMismatchError> {
        if pixels.as_ref().as_ref().len() == get_pixel_count(dimensions) {
            Ok(Self {
                dimensions: dimensions,
                pixels: pixels,
            })
        } else {
            Err(ImageSizeMismatchError { pixels: pixels })
        }
    }
    pub fn get(&self, p: math::Vec2<u32>) -> &Pixel {
        &self.pixels.as_ref().as_ref()[self.get_pixel_offset_in_pixels(p)]
    }
    pub fn get_mut(&mut self, p: math::Vec2<u32>) -> &mut Pixel {
        let index = self.get_pixel_offset_in_pixels(p);
        &mut self.pixels.as_mut().as_mut()[index]
    }
    pub fn into_pixels(self) -> Box<PixelBuffer> {
        self.pixels
    }
    pub fn get_pixels(&self) -> &Box<PixelBuffer> {
        &self.pixels
    }
    pub fn get_mut_pixels(&mut self) -> &mut Box<PixelBuffer> {
        &mut self.pixels
    }
    pub fn copy_from(&mut self, src: &Image) {
        let size = self.dimensions;
        self.copy_area_from(math::Vec2::new(0, 0), src, math::Vec2::new(0, 0), size)
    }
    pub fn copy_area_from(
        &mut self,
        dest_offset: math::Vec2<u32>,
        src: &Image,
        src_offset: math::Vec2<u32>,
        size: math::Vec2<u32>,
    ) {
        for ((((&self_dimension, &dest_offset), &src_dimension), &src_offset), &size) in self
            .dimensions
            .iter()
            .zip(dest_offset.iter())
            .zip(src.dimensions.iter())
            .zip(src_offset.iter())
            .zip(size.iter())
        {
            assert!(dest_offset < self_dimension);
            assert!(dest_offset + size <= self_dimension);
            assert!(src_offset < src_dimension);
            assert!(src_offset + size <= src_dimension);
        }
        let src_pixels = src.pixels.as_ref().as_ref();
        let dest_pixels = self.pixels.as_mut().as_mut();
        for y in 0..size.y {
            for x in 0..size.x {
                let p = math::Vec2::new(x, y);
                let dest_index =
                    get_pixel_offset_in_pixels_unchecked(self.dimensions, p + dest_offset);
                let src_index =
                    get_pixel_offset_in_pixels_unchecked(self.dimensions, p + src_offset);
                dest_pixels[dest_index] = src_pixels[src_index];
            }
        }
    }
    pub fn composite_on_color(&mut self, background_color: Pixel) {
        fn mix(t: u8, a: u8, b: u8) -> u8 {
            let v = (0xFF - t as u32) * a as u32 + t as u32 * b as u32;
            ((v + 0xFF / 2) / 0xFF) as u8
        }
        for pixel in self.pixels.as_mut().as_mut() {
            *pixel = background_color
                .zip(*pixel)
                .map(|(a, b)| mix(pixel.w, a, b));
        }
    }
    pub fn as_ppm(&self, mode: PPMMode) -> ImageAsPPM {
        ImageAsPPM::new(self, mode)
    }
}

pub trait ImageLoader: 'static {
    fn name(&self) -> &'static str;
    fn matches_signature<R: Read>(&self, reader: &mut R) -> io::Result<bool>;
    fn load<R: Read, PBF: PixelBufferFactory>(
        &self,
        reader: &mut R,
        pixel_buffer_factory: PBF,
    ) -> io::Result<Image>;
}
