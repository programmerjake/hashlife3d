mod png;
use super::math;
use std::error;
use std::fmt;
use std::io;
use std::io::prelude::*;

pub struct Image {
    width: u32,
    height: u32,
    pixels: Vec<math::Vec4<u8>>,
}

#[derive(Clone)]
pub struct ImageSizeMismatchError {
    pub pixels: Vec<math::Vec4<u8>>,
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
    pub fn new(width: u32, height: u32, fill_color: math::Vec4<u8>) -> Self {
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
        pixels: Vec<math::Vec4<u8>>,
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
    pub unsafe fn from_pixels_unchecked(
        width: u32,
        height: u32,
        pixels: Vec<math::Vec4<u8>>,
    ) -> Self {
        Self {
            width: width,
            height: height,
            pixels: pixels,
        }
    }
    pub fn get(&self, x: u32, y: u32) -> &math::Vec4<u8> {
        assert!(x < self.width);
        assert!(y < self.height);
        unsafe { self.get_unchecked(x, y) }
    }
    pub unsafe fn get_unchecked(&self, x: u32, y: u32) -> &math::Vec4<u8> {
        self.pixels
            .get_unchecked(self.get_pixel_offset_in_pixels_unchecked(x, y))
    }
    pub fn get_mut(&mut self, x: u32, y: u32) -> &mut math::Vec4<u8> {
        assert!(x < self.width);
        assert!(y < self.height);
        unsafe { self.get_unchecked_mut(x, y) }
    }
    pub unsafe fn get_unchecked_mut(&mut self, x: u32, y: u32) -> &mut math::Vec4<u8> {
        let pixel_offset = self.get_pixel_offset_in_pixels_unchecked(x, y);
        self.pixels.get_unchecked_mut(pixel_offset)
    }
    pub fn into_pixels(self) -> Vec<math::Vec4<u8>> {
        self.pixels
    }
    pub fn get_pixels(&self) -> &Vec<math::Vec4<u8>> {
        &self.pixels
    }
    pub unsafe fn get_mut_pixels(&mut self) -> &mut Vec<math::Vec4<u8>> {
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
}

pub trait ImageLoader: 'static {
    fn name(&self) -> &'static str;
    fn matches_signature<R: Read>(&self, reader: &mut R) -> io::Result<bool>;
    fn load<R: Read>(&self, reader: &mut R) -> io::Result<Image>;
}

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

pub fn load_image<R: Read>(reader: &mut R) -> io::Result<Image> {
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
    struct Visitor<R: Read> {
        result: Option<io::Result<Image>>,
        reader: Option<RestartableReader<R>>,
    }
    impl<R: Read> ImageLoaderVisitor for Visitor<R> {
        fn visit<IL: ImageLoader>(&mut self, image_loader: &IL) -> ImageLoaderVisitorResult {
            let mut reader = self.reader.take().unwrap();
            match image_loader.matches_signature(&mut reader) {
                Ok(true) => {
                    reader.restart();
                    self.result = Some(image_loader.load(&mut reader.into_nonrestartable_reader()));
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

pub fn load_image_bytes(bytes: &[u8]) -> io::Result<Image> {
    load_image(&mut io::Cursor::new(bytes))
}
