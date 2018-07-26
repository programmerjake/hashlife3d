#version 450

layout (location = 0) in vec3 input_position;
layout (location = 1) in vec4 input_color;

layout(push_constant) uniform PushConstants
{
    mat4 transform;
} push_constants;

out gl_PerVertex
{
    vec4 gl_Position;
};

layout (location = 0) out vec4 output_color;

void main()
{
    gl_Position = push_constants.transform * vec4(input_position, 1.0);
    output_color = input_color;
}
