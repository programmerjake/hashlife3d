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
use image;
use renderer::{DeviceReference, StagingImageSet, TextureId};

pub struct ImageResource {
    name: &'static str,
    file_name: &'static str,
    bytes: &'static [u8],
    index: Option<usize>,
}

impl ImageResource {
    pub fn name(&self) -> &'static str {
        self.name
    }
    pub fn file_name(&self) -> &'static str {
        self.file_name
    }
    pub fn tiles_array_index(&self) -> Option<usize> {
        self.index
    }
    pub fn texture_id(&self) -> Option<TextureId> {
        self.index.map(|v| (v + 1) as TextureId)
    }
    pub fn load(&self) -> image::Image {
        image::load_image_bytes(self.bytes, image::DefaultPixelBufferFactory).unwrap()
    }
}

pub mod tiles {
    use super::*;

    pub fn create_tiles_image_set<
        SIS: StagingImageSet,
        DR: DeviceReference<StagingImageSet = SIS>,
    >(
        device_reference: &DR,
    ) -> Result<DR::StagingImageSet, DR::Error> {
        let first_image = TILES_ARRAY[0].load();
        let mut retval = device_reference
            .create_staging_image_set(first_image.dimensions(), TILES_ARRAY.len())?;
        let mut first_image = Some(first_image);
        for tile in TILES_ARRAY {
            let image = first_image.take().unwrap_or_else(|| tile.load());
            assert_eq!(image.dimensions(), retval.dimensions());
            retval.as_mut()[tile.index.unwrap()].copy_from(&image);
        }
        Ok(retval)
    }

    macro_rules! declare_tiles {
        {$($name:ident = $file:expr;)*} => {
            #[allow(non_camel_case_types)]
            enum TilesEnum {
                $($name,)*
            }

            pub const TILES_ARRAY: &'static [ImageResource] = &[
                $(ImageResource {
                    name: stringify!($name),
                    file_name: $file,
                    bytes: include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/images/tiles/",
                        $file
                    )),
                    index: Some(TilesEnum::$name as usize),
                },)*
            ];

            $(pub const $name: &'static ImageResource = &TILES_ARRAY[TilesEnum::$name as usize];)*
        };
    }

    declare_tiles!{
        BEDROCK = "bedrock.png";
        COBBLESTONE = "cobblestone.png";
        GLOWSTONE = "glowstone.png";
        STONE = "stone.png";
        TEST_NX = "test_nx.png";
        TEST_PX = "test_px.png";
        TEST_NY = "test_ny.png";
        TEST_PY = "test_py.png";
        TEST_NZ = "test_nz.png";
        TEST_PZ = "test_pz.png";
    }
}
