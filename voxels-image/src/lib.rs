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
extern crate voxels_image_base as image_base;
extern crate voxels_image_png as png;
extern crate voxels_math as math;
use std::io;
use std::io::prelude::*;

pub use image_base::*;

pub enum ImageLoaderVisitorResult {
    Continue,
    Break,
}

pub trait ImageLoaderVisitor {
    fn visit<IL: ImageLoader>(&mut self, image_loader: &IL) -> ImageLoaderVisitorResult;
}

pub fn visit_image_loaders<ILV: ImageLoaderVisitor>(
    image_loader_visitor: &mut ILV,
) -> ImageLoaderVisitorResult {
    macro_rules! visit_image_loader {
        ($loader:expr) => {
            match image_loader_visitor.visit($loader) {
                ImageLoaderVisitorResult::Continue => {}
                ImageLoaderVisitorResult::Break => return ImageLoaderVisitorResult::Break,
            }
        };
    }
    visit_image_loader!(&png::get_image_loader());
    ImageLoaderVisitorResult::Continue
}

pub fn load_image<R: Read, PBF: PixelBufferFactory>(
    reader: &mut R,
    pixel_buffer_factory: PBF,
) -> io::Result<Image> {
    struct RestartableReader<R: Read> {
        underlying_reader: R,
        buffer: Vec<u8>,
        position: usize,
    }
    impl<R: Read> RestartableReader<R> {
        fn new(underlying_reader: R) -> Self {
            Self {
                underlying_reader: underlying_reader,
                buffer: Vec::new(),
                position: 0,
            }
        }
        fn restart(&mut self) {
            self.position = 0;
        }
        fn into_nonrestartable_reader(self) -> io::Chain<io::Cursor<Vec<u8>>, R> {
            let mut cursor = io::Cursor::new(self.buffer);
            cursor.set_position(self.position as u64);
            cursor.chain(self.underlying_reader)
        }
    }
    impl<R: Read> Read for RestartableReader<R> {
        fn read(&mut self, bytes: &mut [u8]) -> io::Result<usize> {
            if self.position < self.buffer.len() {
                let mut buffer_slice = &self.buffer[self.position..];
                if buffer_slice.len() > bytes.len() {
                    buffer_slice = &buffer_slice[0..bytes.len()];
                }
                bytes[0..buffer_slice.len()].copy_from_slice(buffer_slice);
                self.position += buffer_slice.len();
                return Ok(buffer_slice.len());
            }
            match self.underlying_reader.read(bytes) {
                Ok(count) => {
                    self.buffer.extend_from_slice(&bytes[0..count]);
                    self.position = self.buffer.len();
                    Ok(count)
                }
                Err(error) => Err(error),
            }
        }
    }
    struct Visitor<R: Read, PBF: PixelBufferFactory> {
        result: Option<io::Result<Image>>,
        reader: Option<RestartableReader<R>>,
        pixel_buffer_factory: Option<PBF>,
    }
    impl<R: Read, PBF: PixelBufferFactory> ImageLoaderVisitor for Visitor<R, PBF> {
        fn visit<IL: ImageLoader>(&mut self, image_loader: &IL) -> ImageLoaderVisitorResult {
            let mut reader = self.reader.take().unwrap();
            match image_loader.matches_signature(&mut reader) {
                Ok(true) => {
                    reader.restart();
                    self.result = Some(image_loader.load(
                        &mut reader.into_nonrestartable_reader(),
                        self.pixel_buffer_factory.take().unwrap(),
                    ));
                    ImageLoaderVisitorResult::Break
                }
                Ok(false) => {
                    reader.restart();
                    self.reader = Some(reader);
                    ImageLoaderVisitorResult::Continue
                }
                Err(error) => {
                    self.result = Some(Err(error));
                    ImageLoaderVisitorResult::Break
                }
            }
        }
    }
    let mut visitor = Visitor {
        result: None,
        reader: Some(RestartableReader::new(reader)),
        pixel_buffer_factory: Some(pixel_buffer_factory),
    };
    visit_image_loaders(&mut visitor);
    if let Some(result) = visitor.result {
        result
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unrecognized image format",
        ))
    }
}

pub fn load_image_bytes<PBF: PixelBufferFactory>(
    bytes: &[u8],
    pixel_buffer_factory: PBF,
) -> io::Result<Image> {
    load_image(&mut io::Cursor::new(bytes), pixel_buffer_factory)
}
