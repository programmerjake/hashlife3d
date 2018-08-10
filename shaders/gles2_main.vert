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
