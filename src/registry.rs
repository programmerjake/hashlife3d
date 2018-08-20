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
use block::{BlockDescriptor, BlockId, UninitializedBlock};
use std::collections::{hash_map::Entry, HashMap};
use std::sync::Arc;

struct RegistryData {
    blocks_map: HashMap<&'static str, BlockId>,
    blocks_array: Vec<&'static BlockDescriptor>,
}

pub struct Registry(Arc<RegistryData>);

#[macro_export]
macro_rules! register_texture {
    ($registry_builder:expr, $texture:expr) => {
        $registry_builder.register_texture(
            include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/textures/", $texture)),
            $texture,
        )
    };
}

pub struct RegistryBuilder {
    data: RegistryData,
}

impl RegistryBuilder {
    pub fn new() -> Self {
        let mut retval = Self {
            data: RegistryData {
                blocks_map: HashMap::new(),
                blocks_array: Vec::new(),
            },
        };
        let block_id = retval.register_block(UninitializedBlock::get());
        assert_eq!(block_id, Default::default());
        retval
    }
    pub fn finish_startup(self) -> Registry {
        Registry(Arc::new(self.data))
    }
    pub fn register_block(&mut self, block: &'static BlockDescriptor) -> BlockId {
        use self::Entry::*;
        let block_id = match self.data.blocks_map.entry(block.id_string()) {
            Occupied(_) => panic!("block already registered: {:?}", block),
            Vacant(entry) => {
                let block_id = BlockId::new(self.data.blocks_array.len() as u32);
                self.data.blocks_array.push(block);
                entry.insert(block_id);
                block_id
            }
        };
        block.on_register(block_id, self);
        block_id
    }
}
