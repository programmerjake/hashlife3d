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
mod air;
mod stone;
use registry;

mod block_definition {
    use enum_map::EnumMap;
    use geometry::Mesh;
    use math::{self, Mappable};
    use registry;
    use registry::Registry;
    use std::fmt;
    use std::hash;
    use std::mem;
    use std::u8;

    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct BlockId(u32);

    impl BlockId {
        pub const MAX: BlockId = BlockId(0xFFFFF);
        pub fn new(id: u32) -> Self {
            assert!(id <= Self::MAX.0);
            BlockId(id)
        }
        pub fn value(self) -> u32 {
            self.0
        }
    }

    impl Default for BlockId {
        fn default() -> BlockId {
            BlockId(0)
        }
    }

    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
    pub struct LightLevel(u32);

    impl LightLevel {
        pub const MAX: LightLevel = LightLevel(0xF);
        pub const ZERO: LightLevel = LightLevel(0);
        pub const ONE: LightLevel = LightLevel(1);
        pub fn new(level: u32) -> Self {
            assert!(level <= Self::MAX.0);
            Self { 0: level }
        }
        pub fn get(self) -> u32 {
            self.0
        }
        pub fn reduced(self, reduce_amount: Self) -> Self {
            if self > reduce_amount {
                LightLevel(self.0 - reduce_amount.0)
            } else {
                Self::ZERO
            }
        }
    }

    impl Default for LightLevel {
        fn default() -> LightLevel {
            LightLevel(0)
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct BlockLightProperties {
        pub direct_reduce: LightLevel,
        pub diffuse_reduce: LightLevel,
        pub emissive: LightLevel,
    }

    impl BlockLightProperties {
        pub const AIR: BlockLightProperties = BlockLightProperties {
            direct_reduce: LightLevel::ZERO,
            diffuse_reduce: LightLevel::ONE,
            emissive: LightLevel::ZERO,
        };
        pub const OPAQUE: BlockLightProperties = BlockLightProperties {
            direct_reduce: LightLevel::MAX,
            diffuse_reduce: LightLevel::MAX,
            emissive: LightLevel::ZERO,
        };
        pub fn is_opaque_for_smooth_shading(self) -> bool {
            self.diffuse_reduce > LightLevel(2)
        }
        pub fn propagate_lighting_from(
            self,
            source_lighting: BlockLighting,
            source_face: BlockFace,
        ) -> BlockLighting {
            let direct_reduce = match source_face {
                BlockFace::PY => self.direct_reduce,
                _ => LightLevel::MAX,
            };
            let natural_direct_light_level = source_lighting
                .natural_direct_light_level()
                .reduced(direct_reduce);
            let natural_diffuse_light_level = natural_direct_light_level.max(
                source_lighting
                    .natural_diffuse_light_level()
                    .reduced(self.diffuse_reduce),
            );
            let artificial_diffuse_light_level = self.emissive.max(
                source_lighting
                    .artificial_diffuse_light_level()
                    .reduced(self.diffuse_reduce),
            );
            BlockLighting::new(
                artificial_diffuse_light_level,
                natural_diffuse_light_level,
                natural_direct_light_level,
            )
        }
    }

    impl Default for BlockLightProperties {
        fn default() -> Self {
            Self::OPAQUE
        }
    }

    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct Block(u32);

    impl Default for Block {
        fn default() -> Block {
            Block(0)
        }
    }

    impl Block {
        pub fn id(self) -> BlockId {
            BlockId(self.0 & BlockId::MAX.0)
        }
        pub fn artificial_diffuse_light_level(self) -> LightLevel {
            LightLevel((self.0 >> 20) & 0xF)
        }
        pub fn natural_diffuse_light_level(self) -> LightLevel {
            LightLevel((self.0 >> 24) & 0xF)
        }
        pub fn natural_direct_light_level(self) -> LightLevel {
            LightLevel((self.0 >> 28) & 0xF)
        }
        pub fn lighting(self) -> BlockLighting {
            BlockLighting(Block(self.0 & !BlockId::MAX.0))
        }
        pub fn with_split_lighting(
            id: BlockId,
            artificial_diffuse_light_level: LightLevel,
            natural_diffuse_light_level: LightLevel,
            natural_direct_light_level: LightLevel,
        ) -> Self {
            Block(
                id.0 | (artificial_diffuse_light_level.0 << 20)
                    | (natural_diffuse_light_level.0 << 24)
                    | (natural_direct_light_level.0 << 28),
            )
        }
        pub fn new(id: BlockId, lighting: BlockLighting) -> Self {
            Block(id.0 | lighting.0 .0)
        }
        pub fn with_light_from(id: BlockId, light_source: Self) -> Self {
            Self::new(id, light_source.lighting())
        }
    }

    impl fmt::Debug for Block {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Block")
                .field("id", &self.id())
                .field(
                    "artificial_diffuse_light_level",
                    &self.artificial_diffuse_light_level(),
                ).field(
                    "natural_diffuse_light_level",
                    &self.natural_diffuse_light_level(),
                ).field(
                    "natural_direct_light_level",
                    &self.natural_direct_light_level(),
                ).finish()
        }
    }

    #[repr(transparent)]
    #[derive(Copy, Clone, Eq, PartialEq, Hash)]
    pub struct BlockLighting(Block);

    impl BlockLighting {
        pub fn artificial_diffuse_light_level(self) -> LightLevel {
            self.0.artificial_diffuse_light_level()
        }
        pub fn natural_diffuse_light_level(self) -> LightLevel {
            self.0.natural_diffuse_light_level()
        }
        pub fn natural_direct_light_level(self) -> LightLevel {
            self.0.natural_direct_light_level()
        }
        pub fn new(
            artificial_diffuse_light_level: LightLevel,
            natural_diffuse_light_level: LightLevel,
            natural_direct_light_level: LightLevel,
        ) -> Self {
            BlockLighting(Block::with_split_lighting(
                BlockId(0),
                artificial_diffuse_light_level,
                natural_diffuse_light_level,
                natural_direct_light_level,
            ))
        }
        pub fn max(self, rhs: Self) -> Self {
            Self::new(
                self.artificial_diffuse_light_level()
                    .max(rhs.artificial_diffuse_light_level()),
                self.natural_diffuse_light_level()
                    .max(rhs.natural_diffuse_light_level()),
                self.natural_direct_light_level()
                    .max(rhs.natural_direct_light_level()),
            )
        }
    }

    impl fmt::Debug for BlockLighting {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            f.debug_struct("Block")
                .field(
                    "artificial_diffuse_light_level",
                    &self.artificial_diffuse_light_level(),
                ).field(
                    "natural_diffuse_light_level",
                    &self.natural_diffuse_light_level(),
                ).field(
                    "natural_direct_light_level",
                    &self.natural_direct_light_level(),
                ).finish()
        }
    }

    impl Default for BlockLighting {
        fn default() -> Self {
            BlockLighting(Default::default())
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
    pub struct GlobalLightingProperties {
        pub natural_light_reduce: LightLevel,
    }

    impl Default for GlobalLightingProperties {
        fn default() -> Self {
            Self {
                natural_light_reduce: LightLevel::ZERO,
            }
        }
    }

    impl GlobalLightingProperties {
        pub fn get_light_level(&self, block_lighting: BlockLighting) -> LightLevel {
            block_lighting.artificial_diffuse_light_level().max(
                block_lighting
                    .natural_diffuse_light_level()
                    .reduced(self.natural_light_reduce),
            )
        }
    }

    #[derive(Clone, Copy, Eq, PartialEq, Hash, Debug)]
    pub struct GlobalRenderProperties {
        pub lighting: GlobalLightingProperties,
    }

    impl Default for GlobalRenderProperties {
        fn default() -> Self {
            Self {
                lighting: Default::default(),
            }
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub enum AdjacentBlockFaceVisibility {
        Visible,
        Obscured,
    }

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
    pub struct AdjacentBlockFaceVisibilities {
        pub nx: AdjacentBlockFaceVisibility,
        pub px: AdjacentBlockFaceVisibility,
        pub ny: AdjacentBlockFaceVisibility,
        pub py: AdjacentBlockFaceVisibility,
        pub nz: AdjacentBlockFaceVisibility,
        pub pz: AdjacentBlockFaceVisibility,
    }

    impl AdjacentBlockFaceVisibilities {
        pub const ALL_OBSCURED: AdjacentBlockFaceVisibilities = AdjacentBlockFaceVisibilities {
            nx: AdjacentBlockFaceVisibility::Obscured,
            px: AdjacentBlockFaceVisibility::Obscured,
            ny: AdjacentBlockFaceVisibility::Obscured,
            py: AdjacentBlockFaceVisibility::Obscured,
            nz: AdjacentBlockFaceVisibility::Obscured,
            pz: AdjacentBlockFaceVisibility::Obscured,
        };
        pub const ALL_VISIBLE: AdjacentBlockFaceVisibilities = AdjacentBlockFaceVisibilities {
            nx: AdjacentBlockFaceVisibility::Visible,
            px: AdjacentBlockFaceVisibility::Visible,
            ny: AdjacentBlockFaceVisibility::Visible,
            py: AdjacentBlockFaceVisibility::Visible,
            nz: AdjacentBlockFaceVisibility::Visible,
            pz: AdjacentBlockFaceVisibility::Visible,
        };
    }

    impl From<AdjacentBlockFaceVisibilities> for EnumMap<BlockFace, AdjacentBlockFaceVisibility> {
        fn from(v: AdjacentBlockFaceVisibilities) -> Self {
            Self::from(|block_face| match block_face {
                BlockFace::NX => v.nx,
                BlockFace::PX => v.px,
                BlockFace::NY => v.ny,
                BlockFace::PY => v.py,
                BlockFace::NZ => v.nz,
                BlockFace::PZ => v.pz,
            })
        }
    }

    impl From<EnumMap<BlockFace, AdjacentBlockFaceVisibility>> for AdjacentBlockFaceVisibilities {
        fn from(v: EnumMap<BlockFace, AdjacentBlockFaceVisibility>) -> Self {
            Self {
                nx: v[BlockFace::NX],
                px: v[BlockFace::PX],
                ny: v[BlockFace::NY],
                py: v[BlockFace::PY],
                nz: v[BlockFace::NZ],
                pz: v[BlockFace::PZ],
            }
        }
    }

    #[derive(Copy, Clone, Debug)]
    pub struct BlockProperties {
        pub descriptor: &'static BlockDescriptor,
        pub id_string: &'static str,
        pub light_properties: BlockLightProperties,
        pub adjacent_block_face_visibilities: AdjacentBlockFaceVisibilities,
    }

    pub trait BlockDescriptor: Sync + 'static + fmt::Debug {
        fn get() -> &'static BlockProperties
        where
            Self: Sized;
        fn on_register(&self, block_id: BlockId, registry_builder: &mut registry::RegistryBuilder) {
            mem::drop((block_id, registry_builder));
        }
        fn render(
            &self,
            neighborhood: &[[[Block; 3]; 3]; 3],
            mesh: &mut Mesh,
            position: math::Vec3<i32>,
            global_render_properties: GlobalRenderProperties,
            registry: &Registry,
        );
    }

    pub mod block_render_helpers {
        use block::{
            AdjacentBlockFaceVisibility, Block, BlockFace, BlockRenderLighting,
            GlobalRenderProperties,
        };
        use enum_map::EnumMap;
        use geometry::Mesh;
        use math::{self, Mappable};
        use registry::Registry;
        use renderer::TextureId;

        pub fn render_solid(
            neighborhood: &[[[Block; 3]; 3]; 3],
            mesh: &mut Mesh,
            position: math::Vec3<i32>,
            global_render_properties: GlobalRenderProperties,
            textures: EnumMap<BlockFace, TextureId>,
            registry: &Registry,
        ) {
            let get = |p: math::Vec3<i32>| -> Block {
                let p = p.map(|v| (v + 1) as usize);
                neighborhood[p.x][p.y][p.z]
            };
            let textures = EnumMap::from(|block_face: BlockFace| {
                if EnumMap::<BlockFace, AdjacentBlockFaceVisibility>::from(
                    registry
                        .get_block(get(block_face.into()).id())
                        .adjacent_block_face_visibilities,
                )[block_face.opposite()]
                    == AdjacentBlockFaceVisibility::Obscured
                {
                    return None;
                }
                Some(textures[block_face])
            });
            if let [None, None, None, None, None, None] = textures.as_slice() {
                return;
            }
            let lighting = BlockRenderLighting::from_blocks(
                &neighborhood,
                &global_render_properties.lighting,
                registry,
            );
            for (block_face, texture) in textures {
                if let Some(texture) = texture {
                    mesh.add_cube_face(
                        position.map(|v| v as f32),
                        |vertex_position| {
                            lighting.get_face_vertex_color(
                                vertex_position.map(|v| v as f32),
                                block_face,
                                math::Vec4::splat(1.0),
                            )
                        },
                        texture,
                        block_face,
                    );
                }
            }
            mesh.add_cube(
                position.map(|v| v as f32),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                math::Vec4::splat(0xFF),
                textures[BlockFace::NX],
                textures[BlockFace::PX],
                textures[BlockFace::NY],
                textures[BlockFace::PY],
                textures[BlockFace::NZ],
                textures[BlockFace::PZ],
            )
        }
    }

    #[derive(Copy, Clone, Eq, PartialEq, Hash, Debug, Enum)]
    #[repr(u8)]
    pub enum BlockFace {
        NX = 0,
        PX = 1,
        NY = 2,
        PY = 3,
        NZ = 4,
        PZ = 5,
    }

    impl BlockFace {
        pub fn opposite(self) -> Self {
            match self {
                BlockFace::NX => BlockFace::PX,
                BlockFace::PX => BlockFace::NX,
                BlockFace::NY => BlockFace::PY,
                BlockFace::PY => BlockFace::NY,
                BlockFace::NZ => BlockFace::PZ,
                BlockFace::PZ => BlockFace::NZ,
            }
        }
    }

    impl From<BlockFace> for math::Vec3<i32> {
        fn from(v: BlockFace) -> Self {
            match v {
                BlockFace::NX => math::Vec3::new(-1, 0, 0),
                BlockFace::PX => math::Vec3::new(1, 0, 0),
                BlockFace::NY => math::Vec3::new(0, -1, 0),
                BlockFace::PY => math::Vec3::new(0, 1, 0),
                BlockFace::NZ => math::Vec3::new(0, 0, -1),
                BlockFace::PZ => math::Vec3::new(0, 0, 1),
            }
        }
    }

    impl From<math::Vec3<i32>> for BlockFace {
        fn from(v: math::Vec3<i32>) -> BlockFace {
            match (v.x, v.y, v.z) {
                (-1, 0, 0) => BlockFace::NX,
                (1, 0, 0) => BlockFace::PX,
                (0, -1, 0) => BlockFace::NY,
                (0, 1, 0) => BlockFace::PY,
                (0, 0, -1) => BlockFace::NZ,
                (0, 0, 1) => BlockFace::PZ,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Copy, Clone, PartialEq, Debug)]
    struct BlockRenderLightingFactor(f32);

    impl BlockRenderLightingFactor {
        fn get(self) -> f32 {
            self.0
        }
        fn new(
            lighting: BlockLighting,
            global_lighting_properties: &GlobalLightingProperties,
        ) -> Self {
            BlockRenderLightingFactor(
                global_lighting_properties.get_light_level(lighting).get() as f32
                    / LightLevel::MAX.get() as f32,
            )
        }
    }

    #[derive(Copy, Clone, Debug, PartialEq)]
    pub struct BlockRenderLighting {
        faces: EnumMap<BlockFace, [[BlockRenderLightingFactor; 2]; 2]>,
        center: [[[BlockRenderLightingFactor; 2]; 2]; 2],
    }

    impl BlockRenderLighting {
        pub fn from_blocks(
            neighborhood: &[[[Block; 3]; 3]; 3],
            global_lighting_properties: &GlobalLightingProperties,
            registry: &Registry,
        ) -> Self {
            let mut light_properties = [[[BlockLightProperties::default(); 3]; 3]; 3];
            for (light_properties, blocks) in light_properties.iter_mut().zip(neighborhood) {
                for (light_properties, blocks) in light_properties.iter_mut().zip(blocks) {
                    for (light_properties, block) in light_properties.iter_mut().zip(blocks) {
                        *light_properties = registry.get_block(block.id()).light_properties;
                    }
                }
            }
            let mut new_neighborhood = [[[BlockLighting::default(); 3]; 3]; 3];
            for (new_neighborhood, blocks) in new_neighborhood.iter_mut().zip(neighborhood) {
                for (new_neighborhood, blocks) in new_neighborhood.iter_mut().zip(blocks) {
                    for (new_neighborhood, block) in new_neighborhood.iter_mut().zip(blocks) {
                        *new_neighborhood = block.lighting();
                    }
                }
            }
            Self::from_blocks_with_light_properties(
                &new_neighborhood,
                &light_properties,
                global_lighting_properties,
            )
        }
        pub fn from_blocks_with_light_properties(
            neighborhood: &[[[BlockLighting; 3]; 3]; 3],
            light_properties: &[[[BlockLightProperties; 3]; 3]; 3],
            global_lighting_properties: &GlobalLightingProperties,
        ) -> Self {
            #[derive(Copy, Clone)]
            struct BlockAndLighting {
                block: BlockLighting,
                light_properties: BlockLightProperties,
            }

            #[inline(always)]
            fn calculate_lighting(
                center_x_center_y_center_z: BlockAndLighting,
                center_x_center_y_side_z: BlockAndLighting,
                center_x_side_y_center_z: BlockAndLighting,
                center_x_side_y_side_z: BlockAndLighting,
                side_x_center_y_center_z: BlockAndLighting,
                side_x_center_y_side_z: BlockAndLighting,
                side_x_side_y_center_z: BlockAndLighting,
                side_x_side_y_side_z: BlockAndLighting,
                global_lighting_properties: &GlobalLightingProperties,
            ) -> BlockRenderLightingFactor {
                let mut lighting_cube = [
                    [
                        [center_x_center_y_center_z, center_x_center_y_side_z],
                        [center_x_side_y_center_z, center_x_side_y_side_z],
                    ],
                    [
                        [side_x_center_y_center_z, side_x_center_y_side_z],
                        [side_x_side_y_center_z, side_x_side_y_side_z],
                    ],
                ];
                fn get(
                    lighting_cube: &mut [[[BlockAndLighting; 2]; 2]; 2],
                    index: (usize, usize, usize),
                ) -> &mut BlockAndLighting {
                    &mut lighting_cube[index.0][index.1][index.2]
                }
                for &(from, to) in &[
                    ((1, 1, 1), (1, 1, 0)),
                    ((1, 1, 1), (1, 0, 1)),
                    ((1, 1, 1), (0, 1, 1)),
                    ((1, 1, 0), (1, 0, 0)),
                    ((1, 1, 0), (0, 1, 0)),
                    ((1, 0, 1), (1, 0, 0)),
                    ((1, 0, 1), (0, 0, 1)),
                    ((0, 1, 1), (0, 1, 0)),
                    ((0, 1, 1), (0, 0, 1)),
                    ((0, 0, 1), (0, 0, 0)),
                    ((0, 1, 0), (0, 0, 0)),
                    ((1, 0, 0), (0, 0, 0)),
                ] {
                    if !get(&mut lighting_cube, from)
                        .light_properties
                        .is_opaque_for_smooth_shading()
                    {
                        let from = *get(&mut lighting_cube, from);
                        let to = get(&mut lighting_cube, to);
                        to.block = to.block.max(from.block);
                    }
                }
                BlockRenderLightingFactor::new(
                    get(&mut lighting_cube, (0, 0, 0)).block,
                    global_lighting_properties,
                )
            }
            let get_block_and_lighting = |p: math::Vec3<i32>| -> BlockAndLighting {
                let x = (p.x + 1) as usize;
                let y = (p.y + 1) as usize;
                let z = (p.z + 1) as usize;
                BlockAndLighting {
                    block: neighborhood[x][y][z],
                    light_properties: light_properties[x][y][z],
                }
            };
            let insert_face_axis = |p: math::Vec2<i32>, block_face: BlockFace| match block_face {
                BlockFace::NX => math::Vec3::new(-1, p.x, p.y),
                BlockFace::PX => math::Vec3::new(1, p.x, p.y),
                BlockFace::NY => math::Vec3::new(p.x, -1, p.y),
                BlockFace::PY => math::Vec3::new(p.x, 1, p.y),
                BlockFace::NZ => math::Vec3::new(p.x, p.y, -1),
                BlockFace::PZ => math::Vec3::new(p.x, p.y, 1),
            };
            fn create_square_array<T, F: FnMut(usize, usize) -> T>(mut f: F) -> [[T; 2]; 2] {
                [[f(0, 0), f(0, 1)], [f(1, 0), f(1, 1)]]
            }
            fn create_cube_array<T, F: FnMut(usize, usize, usize) -> T>(
                mut f: F,
            ) -> [[[T; 2]; 2]; 2] {
                [
                    [[f(0, 0, 0), f(0, 0, 1)], [f(0, 1, 0), f(0, 1, 1)]],
                    [[f(1, 0, 0), f(1, 0, 1)], [f(1, 1, 0), f(1, 1, 1)]],
                ]
            }
            fn select<T>(v: math::Vec3<bool>, t: math::Vec3<T>, f: math::Vec3<T>) -> math::Vec3<T> {
                v.zip(t).zip(f).map(|((v, t), f)| if v { t } else { f })
            }
            let calculate_lighting_from_side = |side: math::Vec3<i32>| {
                calculate_lighting(
                    get_block_and_lighting(select(
                        math::Vec3::new(false, false, false),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(false, false, true),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(false, true, false),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(false, true, true),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(true, false, false),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(true, false, true),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(true, true, false),
                        side,
                        math::Vec3::splat(0),
                    )),
                    get_block_and_lighting(select(
                        math::Vec3::new(true, true, true),
                        side,
                        math::Vec3::splat(0),
                    )),
                    global_lighting_properties,
                )
            };
            let faces = EnumMap::from(|block_face| {
                create_square_array(|x, y| {
                    calculate_lighting_from_side(insert_face_axis(
                        math::Vec2::new(x, y).map(|v| if v == 0 { -1 } else { 1 }),
                        block_face,
                    ))
                })
            });
            let center = create_cube_array(|x, y, z| {
                calculate_lighting_from_side(math::Vec3::new(x, y, z).map(|v| {
                    if v == 0 {
                        -1
                    } else {
                        1
                    }
                }))
            });
            Self {
                faces: faces,
                center: center,
            }
        }
        fn interpolate(t: f32, a: f32, b: f32) -> f32 {
            a + (b - a) * t
        }
        fn get_center_vertex_factor(&self, position: math::Vec3<f32>) -> f32 {
            let nynz = Self::interpolate(
                position.x,
                self.center[0][0][0].get(),
                self.center[1][0][0].get(),
            );
            let pynz = Self::interpolate(
                position.x,
                self.center[0][1][0].get(),
                self.center[1][1][0].get(),
            );
            let nypz = Self::interpolate(
                position.x,
                self.center[0][0][1].get(),
                self.center[1][0][1].get(),
            );
            let pypz = Self::interpolate(
                position.x,
                self.center[0][1][1].get(),
                self.center[1][1][1].get(),
            );
            let nz = Self::interpolate(position.y, nynz, pynz);
            let pz = Self::interpolate(position.y, nypz, pypz);
            Self::interpolate(position.z, nz, pz)
        }
        fn get_face_vertex_factor_helper(
            self_face: &[[BlockRenderLightingFactor; 2]; 2],
            position: math::Vec2<f32>,
        ) -> f32 {
            let ny = Self::interpolate(position.x, self_face[0][0].get(), self_face[1][0].get());
            let py = Self::interpolate(position.x, self_face[0][1].get(), self_face[1][1].get());
            Self::interpolate(position.y, ny, py)
        }
        fn get_face_vertex_factor(&self, position: math::Vec3<f32>, block_face: BlockFace) -> f32 {
            match block_face {
                BlockFace::NX | BlockFace::PX => Self::get_face_vertex_factor_helper(
                    &self.faces[block_face],
                    math::Vec2::new(position.y, position.z),
                ),
                BlockFace::NY | BlockFace::PY => Self::get_face_vertex_factor_helper(
                    &self.faces[block_face],
                    math::Vec2::new(position.x, position.z),
                ),
                BlockFace::NZ | BlockFace::PZ => Self::get_face_vertex_factor_helper(
                    &self.faces[block_face],
                    math::Vec2::new(position.x, position.y),
                ),
            }
        }
        fn get_vertex_color_helper(factor: f32, color: math::Vec4<f32>) -> math::Vec4<u8> {
            (color.map(|v| v.max(0.0).min(1.0)) * math::Vec4::splat(factor * u8::MAX as f32))
                .map(|v| v.round() as u8)
        }
        pub fn get_face_vertex_color(
            &self,
            position: math::Vec3<f32>,
            block_face: BlockFace,
            color: math::Vec4<f32>,
        ) -> math::Vec4<u8> {
            Self::get_vertex_color_helper(self.get_face_vertex_factor(position, block_face), color)
        }
        pub fn get_center_vertex_color(
            &self,
            position: math::Vec3<f32>,
            color: math::Vec4<f32>,
        ) -> math::Vec4<u8> {
            Self::get_vertex_color_helper(self.get_center_vertex_factor(position), color)
        }
    }

    impl<'a> hash::Hash for &'a dyn BlockDescriptor {
        fn hash<H: hash::Hasher>(&self, hasher: &mut H) {
            let v: *const dyn BlockDescriptor = *self;
            v.hash(hasher)
        }
    }

    impl<'a> Eq for &'a dyn BlockDescriptor {}

    impl<'a> PartialEq for &'a dyn BlockDescriptor {
        fn eq(&self, rhs: &Self) -> bool {
            let lhs: *const dyn BlockDescriptor = *self;
            let rhs: *const dyn BlockDescriptor = *rhs;
            lhs == rhs
        }
    }

    #[derive(Debug)]
    pub struct UninitializedBlock(());

    impl BlockDescriptor for UninitializedBlock {
        fn get() -> &'static BlockProperties {
            const DESCRIPTOR: UninitializedBlock = UninitializedBlock(());
            const BLOCK: BlockProperties = BlockProperties {
                descriptor: &DESCRIPTOR,
                id_string: "uninitialized",
                light_properties: BlockLightProperties::AIR,
                adjacent_block_face_visibilities: AdjacentBlockFaceVisibilities::ALL_OBSCURED,
            };
            &BLOCK
        }
        fn render(
            &self,
            _neighborhood: &[[[Block; 3]; 3]; 3],
            _mesh: &mut Mesh,
            _position: math::Vec3<i32>,
            _global_render_properties: GlobalRenderProperties,
            _registry: &Registry,
        ) {
        }
    }
}

pub use self::block_definition::*;

pub fn register_blocks(registry_builder: &mut registry::RegistryBuilder) {
    let blocks = [air::Air::get(), stone::Stone::get()];
    for &block in &blocks {
        registry_builder.register_block(block);
    }
}
