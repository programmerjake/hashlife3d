use super::*;
use std::fs;
use std::rc::Rc;
use std::result::Result;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
struct HashedImage {
    width: u32,
    height: u32,
    crc: u32,
}

impl<'a> From<&'a Image> for HashedImage {
    fn from(image: &'a Image) -> Self {
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

#[derive(Debug, Clone)]
struct ImageLoadError(Option<Rc<Error>>);

impl PartialEq for ImageLoadError {
    fn eq(&self, _: &Self) -> bool {
        true
    }
}

fn test_load_png(bytes: &[u8]) -> Result<Image, ImageLoadError> {
    for byte_count in 0..bytes.len() {
        assert!(
            get_image_loader()
                .load(&mut io::Cursor::new(&bytes[..byte_count]))
                .is_err()
        );
    }
    get_image_loader()
        .load(&mut io::Cursor::new(bytes))
        .map_err(|err| ImageLoadError(Some(Rc::new(err))))
}

macro_rules! test_png {
    ($name:ident, $result:expr) => {
        #[test]
        fn $name() {
            let image = test_load_png(include_bytes!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/pngsuite/",
                stringify!($name),
                ".png"
            )));
            let image_hash = match &image {
                Ok(image) => Ok(image.into()),
                Err(error) => Err(error.clone()),
            };
            let expected_result: Result<HashedImage, ImageLoadError> = $result;
            if image_hash != expected_result {
                if let Ok(image) = &image {
                    let file_path = concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/pngsuite_out/",
                        stringify!($name),
                        ".ppm"
                    );
                    eprintln!("Test failed; writing decoded image to {:?}", file_path);
                    let write = || {
                        let mut ppm = Vec::new();
                        image.as_ppm().read_to_end(&mut ppm)?;
                        fs::write(file_path, ppm)
                    };
                    if let Err(error) = write() {
                        eprintln!("Writing decoded image failed: {}", error);
                    }
                }
                assert_eq!(image_hash, expected_result);
            }
        }
    };
}

test_png!(
    basi0g01,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 228755220
    })
);
test_png!(
    basi0g02,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 775938693
    })
);
test_png!(
    basi0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 2366596123
    })
);
test_png!(
    basi0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
/*
test_png!(
    basi2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi3p01,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi3p02,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi4a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi4a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi6a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basi6a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
*/
test_png!(
    basn0g01,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 228755220
    })
);
test_png!(
    basn0g02,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 775938693
    })
);
test_png!(
    basn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 2366596123
    })
);
test_png!(
    basn0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
/*
test_png!(
    basn2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn3p01,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn3p02,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn4a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn4a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn6a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    basn6a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgai4a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgai4a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgan6a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgan6a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgbn4a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bggn4a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgwn6a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    bgyn6a16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ccwn2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ccwn3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cdfn2c08,
    Ok(HashedImage {
        width: 8,
        height: 32,
        crc: 0
    })
);
test_png!(
    cdhn2c08,
    Ok(HashedImage {
        width: 32,
        height: 8,
        crc: 0
    })
);
test_png!(
    cdsn2c08,
    Ok(HashedImage {
        width: 8,
        height: 8,
        crc: 0
    })
);
test_png!(
    cdun2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ch1n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ch2n3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cm0n0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cm7n0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cm9n0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cs3n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cs3n3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cs5n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cs5n3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cs8n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cs8n3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ct0n0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ct1n0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cten0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ctfn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ctgn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    cthn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ctjn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ctzn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    exif2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f00n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f00n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f01n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f01n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f02n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f02n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f03n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f03n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f04n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f04n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    f99n0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g03n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g03n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g03n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g04n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g04n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g04n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g05n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g05n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g05n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g07n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g07n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g07n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g10n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g10n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g10n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g25n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g25n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    g25n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi1n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi1n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi2n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi2n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi4n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi4n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi9n0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    oi9n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    pp0n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    pp0n6a08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ps1n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ps1n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ps2n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    ps2n2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    s01i3p01,
    Ok(HashedImage {
        width: 1,
        height: 1,
        crc: 0
    })
);
test_png!(
    s01n3p01,
    Ok(HashedImage {
        width: 1,
        height: 1,
        crc: 0
    })
);
test_png!(
    s02i3p01,
    Ok(HashedImage {
        width: 2,
        height: 2,
        crc: 0
    })
);
test_png!(
    s02n3p01,
    Ok(HashedImage {
        width: 2,
        height: 2,
        crc: 0
    })
);
test_png!(
    s03i3p01,
    Ok(HashedImage {
        width: 3,
        height: 3,
        crc: 0
    })
);
test_png!(
    s03n3p01,
    Ok(HashedImage {
        width: 3,
        height: 3,
        crc: 0
    })
);
test_png!(
    s04i3p01,
    Ok(HashedImage {
        width: 4,
        height: 4,
        crc: 0
    })
);
test_png!(
    s04n3p01,
    Ok(HashedImage {
        width: 4,
        height: 4,
        crc: 0
    })
);
test_png!(
    s05i3p02,
    Ok(HashedImage {
        width: 5,
        height: 5,
        crc: 0
    })
);
test_png!(
    s05n3p02,
    Ok(HashedImage {
        width: 5,
        height: 5,
        crc: 0
    })
);
test_png!(
    s06i3p02,
    Ok(HashedImage {
        width: 6,
        height: 6,
        crc: 0
    })
);
test_png!(
    s06n3p02,
    Ok(HashedImage {
        width: 6,
        height: 6,
        crc: 0
    })
);
test_png!(
    s07i3p02,
    Ok(HashedImage {
        width: 7,
        height: 7,
        crc: 0
    })
);
test_png!(
    s07n3p02,
    Ok(HashedImage {
        width: 7,
        height: 7,
        crc: 0
    })
);
test_png!(
    s08i3p02,
    Ok(HashedImage {
        width: 8,
        height: 8,
        crc: 0
    })
);
test_png!(
    s08n3p02,
    Ok(HashedImage {
        width: 8,
        height: 8,
        crc: 0
    })
);
test_png!(
    s09i3p02,
    Ok(HashedImage {
        width: 9,
        height: 9,
        crc: 0
    })
);
test_png!(
    s09n3p02,
    Ok(HashedImage {
        width: 9,
        height: 9,
        crc: 0
    })
);
test_png!(
    s32i3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    s32n3p04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    s33i3p04,
    Ok(HashedImage {
        width: 33,
        height: 33,
        crc: 0
    })
);
test_png!(
    s33n3p04,
    Ok(HashedImage {
        width: 33,
        height: 33,
        crc: 0
    })
);
test_png!(
    s34i3p04,
    Ok(HashedImage {
        width: 34,
        height: 34,
        crc: 0
    })
);
test_png!(
    s34n3p04,
    Ok(HashedImage {
        width: 34,
        height: 34,
        crc: 0
    })
);
test_png!(
    s35i3p04,
    Ok(HashedImage {
        width: 35,
        height: 35,
        crc: 0
    })
);
test_png!(
    s35n3p04,
    Ok(HashedImage {
        width: 35,
        height: 35,
        crc: 0
    })
);
test_png!(
    s36i3p04,
    Ok(HashedImage {
        width: 36,
        height: 36,
        crc: 0
    })
);
test_png!(
    s36n3p04,
    Ok(HashedImage {
        width: 36,
        height: 36,
        crc: 0
    })
);
test_png!(
    s37i3p04,
    Ok(HashedImage {
        width: 37,
        height: 37,
        crc: 0
    })
);
test_png!(
    s37n3p04,
    Ok(HashedImage {
        width: 37,
        height: 37,
        crc: 0
    })
);
test_png!(
    s38i3p04,
    Ok(HashedImage {
        width: 38,
        height: 38,
        crc: 0
    })
);
test_png!(
    s38n3p04,
    Ok(HashedImage {
        width: 38,
        height: 38,
        crc: 0
    })
);
test_png!(
    s39i3p04,
    Ok(HashedImage {
        width: 39,
        height: 39,
        crc: 0
    })
);
test_png!(
    s39n3p04,
    Ok(HashedImage {
        width: 39,
        height: 39,
        crc: 0
    })
);
test_png!(
    s40i3p04,
    Ok(HashedImage {
        width: 40,
        height: 40,
        crc: 0
    })
);
test_png!(
    s40n3p04,
    Ok(HashedImage {
        width: 40,
        height: 40,
        crc: 0
    })
);
test_png!(
    tbbn0g04,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbbn2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbbn3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbgn2c16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbgn3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbrn2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbwn0g16,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbwn3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tbyn3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tm3n3p02,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tp0n0g08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tp0n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tp0n3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    tp1n3p08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(xc1n0g08, Err(ImageLoadError(None)));
test_png!(xc9n2c08, Err(ImageLoadError(None)));
test_png!(xcrn0g04, Err(ImageLoadError(None)));
test_png!(xcsn0g01, Err(ImageLoadError(None)));
test_png!(xd0n2c08, Err(ImageLoadError(None)));
test_png!(xd3n2c08, Err(ImageLoadError(None)));
test_png!(xd9n2c08, Err(ImageLoadError(None)));
test_png!(xdtn0g01, Err(ImageLoadError(None)));
test_png!(xhdn0g08, Err(ImageLoadError(None)));
test_png!(xlfn0g04, Err(ImageLoadError(None)));
test_png!(xs1n0g01, Err(ImageLoadError(None)));
test_png!(xs2n0g01, Err(ImageLoadError(None)));
test_png!(xs4n0g01, Err(ImageLoadError(None)));
test_png!(xs7n0g01, Err(ImageLoadError(None)));
test_png!(
    z00n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    z03n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    z06n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
test_png!(
    z09n2c08,
    Ok(HashedImage {
        width: 32,
        height: 32,
        crc: 0
    })
);
*/
