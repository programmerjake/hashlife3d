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
    use geometry::Mesh;
    use math;
    use registry;
    use std::fmt;
    use std::hash;
    use std::mem;
    use std::ops::Deref;

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
        pub fn new(level: u32) -> Self {
            assert!(level <= Self::MAX.0);
            Self { 0: level }
        }
        pub fn get(self) -> u32 {
            self.0
        }
    }

    impl Default for LightLevel {
        fn default() -> LightLevel {
            LightLevel(0)
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
        pub fn new(
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
        pub fn with_light_from(id: BlockId, light_source: Self) -> Self {
            Block(id.0 | (!BlockId::MAX.0 & light_source.0))
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

    pub trait BlockDescriptor: Sync + 'static + fmt::Debug {
        fn get() -> &'static BlockDescriptor
        where
            Self: Sized;
        fn on_register(&self, block_id: BlockId, registry_builder: &mut registry::RegistryBuilder) {
            mem::drop((block_id, registry_builder));
        }
        fn id_string(&self) -> &'static str;
        fn render(
            &self,
            neighborhood: [[[Block; 3]; 3]; 3],
            mesh: &mut Mesh,
            position: math::Vec3<i32>,
        );
    }

    pub struct RegisteredBlockDescriptor {
        block_descriptor: &'static BlockDescriptor,
    }

    impl Deref for RegisteredBlockDescriptor {
        type Target = &'static BlockDescriptor;
        fn deref(&self) -> &&'static BlockDescriptor {
            &self.block_descriptor
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
        fn get() -> &'static BlockDescriptor {
            const BLOCK: UninitializedBlock = UninitializedBlock(());
            &BLOCK
        }
        fn id_string(&self) -> &'static str {
            "uninitialized"
        }
        fn render(
            &self,
            _neighborhood: [[[Block; 3]; 3]; 3],
            _mesh: &mut Mesh,
            _position: math::Vec3<i32>,
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
