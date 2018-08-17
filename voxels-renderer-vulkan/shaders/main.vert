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
#version 450

layout (location = 0) in vec3 input_position;
layout (location = 1) in vec4 input_color;
layout (location = 2) in vec2 input_texture_coord;
layout (location = 3) in uint input_texture_index;

layout(push_constant) uniform PushConstants
{
    mat4 initial_transform;
    mat4 final_transform;
} push_constants;

out gl_PerVertex
{
    vec4 gl_Position;
};

layout (location = 0) out vec4 color;
layout (location = 1) out vec2 texture_coord;
layout (location = 2) out uint texture_index;

void main()
{
    gl_Position = push_constants.final_transform * (push_constants.initial_transform * vec4(input_position, 1.0));
    color = input_color;
    texture_coord = input_texture_coord;
    texture_index = input_texture_index;
}
