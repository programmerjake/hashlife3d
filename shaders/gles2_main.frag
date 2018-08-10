#version 100

varying mediump vec4 color;
varying mediump vec2 texture_coord;
varying mediump float texture_index;

uniform sampler2D samplers[8];

void main()
{
    gl_FragColor = color;
}
