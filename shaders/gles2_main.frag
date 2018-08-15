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

varying mediump vec4 color;
varying mediump vec2 texture_coord;
varying mediump float texture_index;

uniform sampler2D samplers[8];
uniform mediump float sampler_index_scale;
uniform mediump vec2 texture_coord_scale;
uniform mediump vec2 texture_index_scale;

void main()
{
    mediump float tex_index = texture_index;
    lowp vec4 texture_color = vec4(1.0);
    if(tex_index >= 1.0)
    {
        tex_index--;
        tex_index *= sampler_index_scale;
        mediump int sampler_index = int(floor(tex_index));
        tex_index = fract(tex_index);
        mediump vec2 tex_coord = fract(texture_coord);
        tex_index *= texture_index_scale.y;
        tex_coord.y += floor(tex_index);
        tex_coord.x += fract(tex_index) * texture_index_scale.x;
        tex_coord *= texture_coord_scale;
        if(sampler_index >= 4)
        {
            if(sampler_index >= 6)
            {
                if(sampler_index >= 7)
                    texture_color = texture2D(samplers[7], tex_coord);
                else
                    texture_color = texture2D(samplers[6], tex_coord);
            }
            else
            {
                if(sampler_index >= 5)
                    texture_color = texture2D(samplers[5], tex_coord);
                else
                    texture_color = texture2D(samplers[4], tex_coord);
            }
        }
        else
        {
            if(sampler_index >= 2)
            {
                if(sampler_index >= 3)
                    texture_color = texture2D(samplers[3], tex_coord);
                else
                    texture_color = texture2D(samplers[2], tex_coord);
            }
            else
            {
                if(sampler_index >= 1)
                    texture_color = texture2D(samplers[1], tex_coord);
                else
                    texture_color = texture2D(samplers[0], tex_coord);
            }
        }
    }
    gl_FragColor = color * texture_color;
}
