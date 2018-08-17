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
#version 100

attribute highp vec3 input_position;
attribute mediump vec4 input_color;
attribute mediump vec2 input_texture_coord;
attribute mediump float input_texture_index;

uniform highp mat4 initial_transform;
uniform highp mat4 final_transform;

varying mediump vec4 color;
varying mediump vec2 texture_coord;
varying mediump float texture_index;

void main()
{
    gl_Position = final_transform * (initial_transform * vec4(input_position, 1.0));
    color = input_color;
    texture_coord = input_texture_coord;
    texture_index = input_texture_index;
}
