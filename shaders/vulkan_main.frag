#version 450

layout (location = 0) in vec4 color;
layout (location = 1) in vec2 texture_coord;
layout (location = 2) in flat uint texture_index;

layout (location = 0) out vec4 output_color;

layout (binding = 0) uniform sampler2D samplers[8];

void main()
{
    output_color = color;
}
