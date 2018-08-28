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
use math;
use renderer::{
    DeviceReference, IndexBufferElement, StagingGenericArray, TextureId, VertexBufferElement,
};

pub struct Mesh {
    vertices: Vec<VertexBufferElement>,
    indices: Vec<IndexBufferElement>,
}

impl Mesh {
    pub fn new() -> Self {
        Self {
            vertices: Vec::new(),
            indices: Vec::new(),
        }
    }
    pub fn create_vertex(&mut self, vertex: VertexBufferElement) -> IndexBufferElement {
        let retval = self.vertices.len();
        assert!(retval <= IndexBufferElement::max_value() as usize);
        self.vertices.push(vertex);
        retval as IndexBufferElement
    }
    pub fn add_indexed_triangle(
        &mut self,
        v1: IndexBufferElement,
        v2: IndexBufferElement,
        v3: IndexBufferElement,
    ) {
        self.indices.reserve(3);
        self.indices.push(v1);
        self.indices.push(v2);
        self.indices.push(v3);
    }
    pub fn add_indexed_quad(
        &mut self,
        v1: IndexBufferElement,
        v2: IndexBufferElement,
        v3: IndexBufferElement,
        v4: IndexBufferElement,
    ) {
        self.indices.reserve(6);
        self.add_indexed_triangle(v1, v2, v3);
        self.add_indexed_triangle(v1, v3, v4);
    }
    pub fn add_triangle(
        &mut self,
        v1: VertexBufferElement,
        v2: VertexBufferElement,
        v3: VertexBufferElement,
    ) {
        let v1 = self.create_vertex(v1);
        let v2 = self.create_vertex(v2);
        let v3 = self.create_vertex(v3);
        self.add_indexed_triangle(v1, v2, v3);
    }
    pub fn add_quad(
        &mut self,
        v1: VertexBufferElement,
        v2: VertexBufferElement,
        v3: VertexBufferElement,
        v4: VertexBufferElement,
    ) {
        let v1 = self.create_vertex(v1);
        let v2 = self.create_vertex(v2);
        let v3 = self.create_vertex(v3);
        let v4 = self.create_vertex(v4);
        self.add_indexed_quad(v1, v2, v3, v4);
    }
    pub fn add_mesh(&mut self, mesh: &Mesh) {
        let index_offset = self.vertices.len();
        assert!(index_offset + mesh.indices.len() <= IndexBufferElement::max_value() as usize + 1);
        self.vertices.extend_from_slice(&mesh.vertices);
        self.indices.reserve(mesh.indices.len());
        self.indices.extend(
            mesh.indices
                .iter()
                .map(|&index| index + index_offset as IndexBufferElement),
        );
    }
    pub fn add_cube_face_negative_x(
        &mut self,
        origin: math::Vec3<f32>,
        nxnynz_color: math::Vec4<u8>,
        nxnypz_color: math::Vec4<u8>,
        nxpynz_color: math::Vec4<u8>,
        nxpypz_color: math::Vec4<u8>,
        texture_id: TextureId,
    ) {
        self.add_quad(
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 0.0, 0.0),
                nxnynz_color,
                math::Vec2::new(0.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 0.0, 1.0),
                nxnypz_color,
                math::Vec2::new(1.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 1.0, 1.0),
                nxpypz_color,
                math::Vec2::new(1.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 1.0, 0.0),
                nxpynz_color,
                math::Vec2::new(0.0, 0.0),
                texture_id,
            ),
        )
    }
    pub fn add_cube_face_positive_x(
        &mut self,
        origin: math::Vec3<f32>,
        pxnynz_color: math::Vec4<u8>,
        pxnypz_color: math::Vec4<u8>,
        pxpynz_color: math::Vec4<u8>,
        pxpypz_color: math::Vec4<u8>,
        texture_id: TextureId,
    ) {
        self.add_quad(
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 1.0, 1.0),
                pxpypz_color,
                math::Vec2::new(0.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 0.0, 1.0),
                pxnypz_color,
                math::Vec2::new(0.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 0.0, 0.0),
                pxnynz_color,
                math::Vec2::new(1.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 1.0, 0.0),
                pxpynz_color,
                math::Vec2::new(1.0, 0.0),
                texture_id,
            ),
        )
    }
    pub fn add_cube_face_negative_y(
        &mut self,
        origin: math::Vec3<f32>,
        nxnynz_color: math::Vec4<u8>,
        nxnypz_color: math::Vec4<u8>,
        pxnynz_color: math::Vec4<u8>,
        pxnypz_color: math::Vec4<u8>,
        texture_id: TextureId,
    ) {
        self.add_quad(
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 0.0, 0.0),
                nxnynz_color,
                math::Vec2::new(0.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 0.0, 0.0),
                pxnynz_color,
                math::Vec2::new(1.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 0.0, 1.0),
                pxnypz_color,
                math::Vec2::new(1.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 0.0, 1.0),
                nxnypz_color,
                math::Vec2::new(0.0, 0.0),
                texture_id,
            ),
        )
    }
    pub fn add_cube_face_positive_y(
        &mut self,
        origin: math::Vec3<f32>,
        nxpynz_color: math::Vec4<u8>,
        nxpypz_color: math::Vec4<u8>,
        pxpynz_color: math::Vec4<u8>,
        pxpypz_color: math::Vec4<u8>,
        texture_id: TextureId,
    ) {
        self.add_quad(
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 1.0, 1.0),
                pxpypz_color,
                math::Vec2::new(1.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 1.0, 0.0),
                pxpynz_color,
                math::Vec2::new(1.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 1.0, 0.0),
                nxpynz_color,
                math::Vec2::new(0.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 1.0, 1.0),
                nxpypz_color,
                math::Vec2::new(0.0, 1.0),
                texture_id,
            ),
        )
    }
    pub fn add_cube_face_negative_z(
        &mut self,
        origin: math::Vec3<f32>,
        nxnynz_color: math::Vec4<u8>,
        nxpynz_color: math::Vec4<u8>,
        pxnynz_color: math::Vec4<u8>,
        pxpynz_color: math::Vec4<u8>,
        texture_id: TextureId,
    ) {
        self.add_quad(
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 0.0, 0.0),
                nxnynz_color,
                math::Vec2::new(1.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 1.0, 0.0),
                nxpynz_color,
                math::Vec2::new(1.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 1.0, 0.0),
                pxpynz_color,
                math::Vec2::new(0.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 0.0, 0.0),
                pxnynz_color,
                math::Vec2::new(0.0, 1.0),
                texture_id,
            ),
        )
    }
    pub fn add_cube_face_positive_z(
        &mut self,
        origin: math::Vec3<f32>,
        nxnypz_color: math::Vec4<u8>,
        nxpypz_color: math::Vec4<u8>,
        pxnypz_color: math::Vec4<u8>,
        pxpypz_color: math::Vec4<u8>,
        texture_id: TextureId,
    ) {
        self.add_quad(
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 1.0, 1.0),
                pxpypz_color,
                math::Vec2::new(1.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 1.0, 1.0),
                nxpypz_color,
                math::Vec2::new(0.0, 0.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(0.0, 0.0, 1.0),
                nxnypz_color,
                math::Vec2::new(0.0, 1.0),
                texture_id,
            ),
            VertexBufferElement::new(
                origin + math::Vec3::new(1.0, 0.0, 1.0),
                pxnypz_color,
                math::Vec2::new(1.0, 1.0),
                texture_id,
            ),
        )
    }
    pub fn add_cube(
        &mut self,
        origin: math::Vec3<f32>,
        nxnynz_color: math::Vec4<u8>,
        nxnypz_color: math::Vec4<u8>,
        nxpynz_color: math::Vec4<u8>,
        nxpypz_color: math::Vec4<u8>,
        pxnynz_color: math::Vec4<u8>,
        pxnypz_color: math::Vec4<u8>,
        pxpynz_color: math::Vec4<u8>,
        pxpypz_color: math::Vec4<u8>,
        nx_face_texture_id: Option<TextureId>,
        px_face_texture_id: Option<TextureId>,
        ny_face_texture_id: Option<TextureId>,
        py_face_texture_id: Option<TextureId>,
        nz_face_texture_id: Option<TextureId>,
        pz_face_texture_id: Option<TextureId>,
    ) {
        if let Some(texture_id) = nx_face_texture_id {
            self.add_cube_face_negative_x(
                origin,
                nxnynz_color,
                nxnypz_color,
                nxpynz_color,
                nxpypz_color,
                texture_id,
            );
        }
        if let Some(texture_id) = px_face_texture_id {
            self.add_cube_face_positive_x(
                origin,
                pxnynz_color,
                pxnypz_color,
                pxpynz_color,
                pxpypz_color,
                texture_id,
            );
        }
        if let Some(texture_id) = ny_face_texture_id {
            self.add_cube_face_negative_y(
                origin,
                nxnynz_color,
                nxnypz_color,
                pxnynz_color,
                pxnypz_color,
                texture_id,
            );
        }
        if let Some(texture_id) = py_face_texture_id {
            self.add_cube_face_positive_y(
                origin,
                nxpynz_color,
                nxpypz_color,
                pxpynz_color,
                pxpypz_color,
                texture_id,
            );
        }
        if let Some(texture_id) = nz_face_texture_id {
            self.add_cube_face_negative_z(
                origin,
                nxnynz_color,
                nxpynz_color,
                pxnynz_color,
                pxpynz_color,
                texture_id,
            );
        }
        if let Some(texture_id) = pz_face_texture_id {
            self.add_cube_face_positive_z(
                origin,
                nxnypz_color,
                nxpypz_color,
                pxnypz_color,
                pxpypz_color,
                texture_id,
            );
        }
    }
    pub fn create_staging_buffers<DR: DeviceReference>(
        &self,
        device_reference: &DR,
    ) -> Result<(DR::StagingVertexBuffer, DR::StagingIndexBuffer), DR::Error> {
        let vertex_buffer = device_reference.create_staging_vertex_buffer(self.vertices.len())?;
        let index_buffer = device_reference.create_staging_index_buffer(self.indices.len())?;
        vertex_buffer.write().copy_from_slice(&self.vertices);
        index_buffer.write().copy_from_slice(&self.indices);
        Ok((vertex_buffer, index_buffer))
    }
}
