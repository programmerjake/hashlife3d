use super::*;
use std::result::Result;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct HashedImage {
    width: u32,
    height: u32,
    crc: u32,
}

impl From<Image> for HashedImage {
    fn from(image: Image) -> Self {
        let mut hasher = CrcHasher::default();
        for pixel in image.get_pixels() {
            let pixel: [u8; 4] = pixel.into();
            hasher.write(&pixel);
        }
        Self {
            width: image.width(),
            height: image.height(),
            crc: hasher.finish() as u32,
        }
    }
}

#[derive(Debug)]
struct ImageLoadError(Option<Error>);

impl PartialEq for ImageLoadError {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

fn test_load_png(bytes: &[u8]) -> Result<HashedImage, ImageLoadError> {
    for byte_count in 0..bytes.len() {
        assert!(
            get_image_loader()
                .load(&mut io::Cursor::new(&bytes[..byte_count]))
                .is_err()
        );
    }
    get_image_loader()
        .load(&mut io::Cursor::new(bytes))
        .map(Into::into)
        .map_err(|err| ImageLoadError(Some(err)))
}

macro_rules! test_png {
    ($name:ident, $result:expr) => {
        #[test]
        fn $name() {
            assert_eq!(
                test_load_png(include_bytes!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/pngsuite/",
                    stringify!($name),
                    ".png"
                ))),
                $result
            );
        }
    };
}

test_png!(
    basi0g01,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
