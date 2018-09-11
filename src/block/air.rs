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
    AdjacentBlockFaceVisibilities, Block, BlockDescriptor, BlockLightProperties, BlockProperties,
    GlobalRenderProperties,
};
use geometry::Mesh;
use math;
use registry::Registry;

#[derive(Debug)]
pub struct Air(());

impl BlockDescriptor for Air {
    fn get() -> &'static BlockProperties {
        const DESCRIPTOR: Air = Air(());
        const BLOCK: BlockProperties = BlockProperties {
            descriptor: &DESCRIPTOR,
            id_string: "voxels:air",
            light_properties: BlockLightProperties::AIR,
            adjacent_block_face_visibilities: AdjacentBlockFaceVisibilities::ALL_VISIBLE,
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
