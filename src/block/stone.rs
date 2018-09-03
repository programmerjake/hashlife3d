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

use block::Block;
use block::BlockDescriptor;
use geometry::Mesh;
use math::{self, Mappable};
use resources::images::tiles;

#[derive(Debug)]
pub struct Stone(());

impl BlockDescriptor for Stone {
    fn get() -> &'static BlockDescriptor {
        const BLOCK: Stone = Stone(());
        &BLOCK
    }
    fn id_string(&self) -> &'static str {
        "voxels:stone"
    }
    fn render(
        &self,
        neighborhood: [[[Block; 3]; 3]; 3],
        mesh: &mut Mesh,
        position: math::Vec3<i32>,
    ) {
        println!("Stone::render");
        // FIXME: handle neighborhood
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
            tiles::STONE.texture_id(),
            tiles::STONE.texture_id(),
            tiles::STONE.texture_id(),
            tiles::STONE.texture_id(),
            tiles::STONE.texture_id(),
            tiles::STONE.texture_id(),
        )
    }
}
