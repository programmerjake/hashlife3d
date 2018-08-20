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

use block::BlockDescriptor;

#[derive(Debug)]
pub struct Air(());

impl BlockDescriptor for Air {
    fn get() -> &'static BlockDescriptor {
        const BLOCK: Air = Air(());
        &BLOCK
    }
    fn id_string(&self) -> &'static str {
        "voxels:air"
    }
}
