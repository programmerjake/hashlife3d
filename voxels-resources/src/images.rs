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
use math;
use renderer::{DeviceReference, ImageSet, StagingGenericArray, StagingImageSet, TextureId};
use std::borrow::Cow;

#[derive(Clone)]
pub struct ImageResource<'a> {
    name: Cow<'a, str>,
    file_name: Cow<'a, str>,
    bytes: Cow<'a, [u8]>,
    index: Option<usize>,
}

impl<'a> ImageResource<'a> {
    pub fn name(&self) -> &Cow<'a, str> {
        &self.name
    }
    pub fn file_name(&self) -> &Cow<'a, str> {
        &self.file_name
    }
    pub fn tiles_array_index(&self) -> Option<usize> {
        self.index
    }
    pub fn texture_id(&self) -> Option<TextureId> {
        self.index.map(|v| (v + 1) as TextureId)
    }
    pub fn bytes(&self) -> &Cow<'a, [u8]> {
        &self.bytes
    }
    pub fn load(&self) -> image::Image {
        image::load_image_bytes(&*self.bytes, image::DefaultPixelBufferFactory).unwrap()
    }
    pub fn into_owned(self) -> ImageResource<'static> {
        ImageResource {
            name: self.name.into_owned().into(),
            file_name: self.file_name.into_owned().into(),
            bytes: self.bytes.into_owned().into(),
            index: self.index,
        }
    }
    pub fn borrow<'b>(&'b self) -> ImageResource<'b> {
        ImageResource {
            name: Cow::Borrowed(&*self.name),
            file_name: Cow::Borrowed(&*self.file_name),
            bytes: Cow::Borrowed(&*self.bytes),
            index: self.index,
        }
    }
}

pub mod tiles {
    use super::*;

    #[derive(Clone, Serialize, Deserialize)]
    struct TilesImageSetItem {
        name: Cow<'static, str>,
        bytes: Cow<'static, [u8]>,
    }

    #[derive(Clone, Serialize, Deserialize)]
    pub struct TilesImageSet(Vec<TilesImageSetItem>);

    impl TilesImageSet {
        pub fn new() -> Self {
            TilesImageSet(
                TILES_ARRAY
                    .iter()
                    .enumerate()
                    .map(|(index, image_resource)| {
                        assert_eq!(Some(index), image_resource.tiles_array_index());
                        TilesImageSetItem {
                            name: image_resource.name().clone(),
                            bytes: image_resource.bytes().clone(),
                        }
                    }).collect(),
            )
        }
        pub fn create_staging_image_set<DR: DeviceReference>(
            &self,
            device_reference: &DR,
        ) -> Result<DR::StagingImageSet, DR::Error> {
            if self.0.is_empty() {
                return device_reference.create_staging_image_set(math::Vec2::splat(1), 0);
            }
            let first_image =
                image::load_image_bytes(&*self.0[0].bytes, image::DefaultPixelBufferFactory)?;
            let retval = device_reference
                .create_staging_image_set(first_image.dimensions(), self.0.len())?;
            {
                let mut staging_image_set = retval.write();
                let mut first_image = Some(first_image);
                for (index, tile) in self.0.iter().enumerate() {
                    let image = first_image.take().map_or_else(
                        || image::load_image_bytes(&*tile.bytes, image::DefaultPixelBufferFactory),
                        Ok,
                    )?;
                    assert_eq!(image.dimensions(), retval.dimensions());
                    staging_image_set[index].copy_from(&image);
                }
            }
            Ok(retval)
        }
    }

    impl Default for TilesImageSet {
        fn default() -> Self {
            Self::new()
        }
    }

    /*
    pub fn create_tiles_image_set<
        SIS: StagingImageSet,
        DR: DeviceReference<StagingImageSet = SIS>,
    >(
        device_reference: &DR,
    ) -> Result<DR::StagingImageSet, DR::Error> {
        let first_image = TILES_ARRAY[0].load();
        let retval = device_reference
            .create_staging_image_set(first_image.dimensions(), TILES_ARRAY.len())?;
        {
            let mut staging_image_set = retval.write();
            let mut first_image = Some(first_image);
            for tile in TILES_ARRAY {
                let image = first_image.take().unwrap_or_else(|| tile.load());
                assert_eq!(image.dimensions(), retval.dimensions());
                staging_image_set[tile.index.unwrap()].copy_from(&image);
            }
        }
        Ok(retval)
    }*/

    macro_rules! declare_tiles {
        {$($name:ident = $file:expr;)*} => {
            #[allow(non_camel_case_types)]
            enum TilesEnum {
                $($name,)*
            }

            pub const TILES_ARRAY: &'static [ImageResource<'static>] = &[
                $(ImageResource {
                    name: Cow::Borrowed(stringify!($name)),
                    file_name: Cow::Borrowed($file),
                    bytes: Cow::Borrowed(include_bytes!(concat!(
                        env!("CARGO_MANIFEST_DIR"),
                        "/images/tiles/",
                        $file
                    ))),
                    index: Some(TilesEnum::$name as usize),
                },)*
            ];

            $(pub const $name: &'static ImageResource<'static> = &TILES_ARRAY[TilesEnum::$name as usize];)*
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
