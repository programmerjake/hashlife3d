#version 450

#if 0 // FIXME: change back

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

#if 0 // FIXME: change back
layout (location = 0) out vec4 color;
layout (location = 1) out vec2 texture_coord;
layout (location = 2) out uint texture_index;
#endif

void main()
{
#if 0 // FIXME: change back
    gl_Position = push_constants.final_transform * (push_constants.initial_transform * vec4(input_position, 1.0));
    color = input_color;
    texture_coord = input_texture_coord;
#else
    gl_Position = vec4(vec2[](vec2(0.0, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5))[gl_VertexIndex], 0.0, 1.0);
#if 0
    color = vec4(1.0, 1.0, 1.0, 1.0);
    texture_coord = vec2(0.0, 0.0);
#endif
#endif
}
#else

out gl_PerVertex
{
    vec4 gl_Position;
};

void main()
{
    gl_Position = vec4(vec2[](vec2(0.0, -0.5), vec2(0.5, 0.5), vec2(-0.5, 0.5))[gl_VertexIndex], 0.0, 1.0);
}

#endif
