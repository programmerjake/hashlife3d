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
use std::error;
use std::fmt;
use std::io;
use std::io::prelude::*;

pub type Pixel = math::Vec4<u8>;

#[derive(Clone)]
pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}

#[derive(Clone)]
pub struct ImageSizeMismatchError {
    pub pixels: Vec<Pixel>,
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
                    image.width(),
                    image.height()
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
                    if buf.len() != 0 && self.image.width() != 0 && self.y < self.image.height() {
                        self.header_or_scanline.set_position(0);
                        match self.mode {
                            PPMMode::Binary => {
                                let scanline = self.header_or_scanline.get_mut();
                                scanline.resize(self.image.width() as usize * 3, 0);
                                for x in 0..self.image.width() {
                                    let pixel = self.image.get(x, self.y);
                                    scanline[x as usize * 3] = pixel.x;
                                    scanline[x as usize * 3 + 1] = pixel.y;
                                    scanline[x as usize * 3 + 2] = pixel.z;
                                }
                            }
                            PPMMode::Text => {
                                self.header_or_scanline.get_mut().clear();
                                for x in 0..self.image.width() {
                                    let pixel = self.image.get(x, self.y);
                                    write!(
                                        &mut self.header_or_scanline,
                                        "{} {} {} ",
                                        pixel.x, pixel.y, pixel.z,
                                    );
                                }
                                writeln!(&mut self.header_or_scanline);
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

impl Image {
    pub fn width(&self) -> u32 {
        self.width
    }
    pub fn height(&self) -> u32 {
        self.height
    }
    pub fn get_line_offset_in_pixels(&self, y: u32) -> usize {
        assert!(y < self.height);
        unsafe { self.get_line_offset_in_pixels_unchecked(y) }
    }
    pub fn get_pixel_offset_in_pixels(&self, x: u32, y: u32) -> usize {
        assert!(x < self.width);
        assert!(y < self.height);
        unsafe { self.get_pixel_offset_in_pixels_unchecked(x, y) }
    }
    pub unsafe fn get_line_offset_in_pixels_unchecked(&self, y: u32) -> usize {
        self.width as usize * y as usize
    }
    pub unsafe fn get_pixel_offset_in_pixels_unchecked(&self, x: u32, y: u32) -> usize {
        x as usize + self.get_line_offset_in_pixels_unchecked(y)
    }
    pub fn new(width: u32, height: u32, fill_color: Pixel) -> Self {
        let mut pixels = Vec::new();
        pixels.resize(
            (width as usize).checked_mul(height as usize).unwrap(),
            fill_color,
        );
        Self {
            width: width,
            height: height,
            pixels: pixels,
        }
    }
    pub fn from_pixels(
        width: u32,
        height: u32,
        pixels: Vec<Pixel>,
    ) -> Result<Self, ImageSizeMismatchError> {
        if Some(pixels.len()) == (width as usize).checked_mul(height as usize) {
            Ok(Self {
                width: width,
                height: height,
                pixels: pixels,
            })
        } else {
            Err(ImageSizeMismatchError { pixels: pixels })
        }
    }
    pub unsafe fn from_pixels_unchecked(width: u32, height: u32, pixels: Vec<Pixel>) -> Self {
        Self {
            width: width,
            height: height,
            pixels: pixels,
        }
    }
    pub fn get(&self, x: u32, y: u32) -> &Pixel {
        assert!(x < self.width);
        assert!(y < self.height);
        unsafe { self.get_unchecked(x, y) }
    }
    pub unsafe fn get_unchecked(&self, x: u32, y: u32) -> &Pixel {
        self.pixels
            .get_unchecked(self.get_pixel_offset_in_pixels_unchecked(x, y))
    }
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut Pixel {
        assert!(x < self.width);
        assert!(y < self.height);
        unsafe { self.get_unchecked_mut(x, y) }
    }
    pub unsafe fn get_unchecked_mut(&mut self, x: u32, y: u32) -> &mut Pixel {
        let pixel_offset = self.get_pixel_offset_in_pixels_unchecked(x, y);
        self.pixels.get_unchecked_mut(pixel_offset)
    }
    pub fn into_pixels(self) -> Vec<Pixel> {
        self.pixels
    }
    pub fn get_pixels(&self) -> &Vec<Pixel> {
        &self.pixels
    }
    pub unsafe fn get_mut_pixels(&mut self) -> &mut Vec<Pixel> {
        &mut self.pixels
    }
    pub fn copy_area_from(
        &mut self,
        dest_x: u32,
        dest_y: u32,
        src: &Image,
        src_x: u32,
        src_y: u32,
        width: u32,
        height: u32,
    ) {
        assert!(dest_x < self.width);
        assert!(dest_y < self.height);
        assert!(dest_x + width <= self.width);
        assert!(dest_y + height <= self.height);
        assert!(src_x < src.width);
        assert!(src_y < src.height);
        assert!(src_x + width <= src.width);
        assert!(src_y + height <= src.height);
        unsafe {
            self.copy_area_from_unchecked(dest_x, dest_y, src, src_x, src_y, width, height);
        }
    }
    pub unsafe fn copy_area_from_unchecked(
        &mut self,
        dest_x: u32,
        dest_y: u32,
        src: &Image,
        src_x: u32,
        src_y: u32,
        width: u32,
        height: u32,
    ) {
        for y in 0..height {
            for x in 0..width {
                *self.get_unchecked_mut(x + dest_x, y + dest_y) =
                    *src.get_unchecked(x + src_x, y + src_y);
            }
        }
    }
    pub fn composite_on_color(&mut self, background_color: Pixel) {
        fn mix(t: u8, a: u8, b: u8) -> u8 {
            let v = (0xFF - t as u32) * a as u32 + t as u32 * b as u32;
            ((v + 0xFF / 2) / 0xFF) as u8
        }
        for pixel in &mut self.pixels {
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
    fn load<R: Read>(&self, reader: &mut R) -> io::Result<Image>;
}
