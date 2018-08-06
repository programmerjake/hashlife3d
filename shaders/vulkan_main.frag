#version 450

#if 0 // FIXME: change back
layout (location = 0) in vec4 color;
layout (location = 1) in vec2 texture_coord;
layout (location = 2) in flat uint texture_index;
#endif

layout (location = 0) out vec4 output_color;

// FIXME: change back to having samplers
// layout (binding = 0) uniform sampler2D samplers[8];

void main()
{
#if 0 // FIXME: change back
    output_color = color;
#else
    output_color = vec4(0.0, 1.0, 0.0, 1.0);
#endif
}
