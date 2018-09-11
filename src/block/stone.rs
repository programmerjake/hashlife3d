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

use block::{
    block_render_helpers, AdjacentBlockFaceVisibilities, Block, BlockDescriptor,
    BlockLightProperties, BlockProperties, GlobalRenderProperties,
};
use geometry::Mesh;
use math;
use registry::Registry;
use resources::images::tiles;

#[derive(Debug)]
pub struct Stone(());

impl BlockDescriptor for Stone {
    fn get() -> &'static BlockProperties {
        const DESCRIPTOR: Stone = Stone(());
        const BLOCK: BlockProperties = BlockProperties {
            descriptor: &DESCRIPTOR,
            id_string: "voxels:stone",
            light_properties: BlockLightProperties::OPAQUE,
            adjacent_block_face_visibilities: AdjacentBlockFaceVisibilities::ALL_OBSCURED,
        };
        &BLOCK
    }
    fn render(
        &self,
        neighborhood: &[[[Block; 3]; 3]; 3],
        mesh: &mut Mesh,
        position: math::Vec3<i32>,
        global_render_properties: GlobalRenderProperties,
        registry: &Registry,
    ) {
        block_render_helpers::render_solid(
            neighborhood,
            mesh,
            position,
            global_render_properties,
            (|_| tiles::STONE.texture_id().unwrap()).into(),
            registry,
        )
    }
}
