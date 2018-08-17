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

layout (location = 0) in vec4 color;
layout (location = 1) in vec2 texture_coord;
layout (location = 2) in flat uint texture_index;

layout (location = 0) out vec4 output_color;

layout (binding = 0) uniform sampler2DArray samplers[8];

void main()
{
    vec4 texture_color = vec4(1.0);
    if(texture_index != 0)
    {
        uint i = texture_index - 1;
        uint texture_size = textureSize(samplers[0], 0).z;
        uint sampler_index = i / texture_size;
        i %= texture_size;
        texture_color = texture(samplers[sampler_index], vec3(texture_coord, float(i)));
    }
    output_color = color * texture_color;
}
