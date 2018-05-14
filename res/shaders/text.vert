#version 330
layout(location = 0) in vec4 bound;
layout(location = 1) in vec4 uv_bound;
layout(location = 2) in vec4 color;

uniform vec4 viewport;

out GS_IN {
    flat float w;
    flat float h;
    flat vec4 uv_bound;
    flat vec4 color;
} vs_out;

void main() {
    vs_out.color = color;
    float x = -1 + 2 * (bound.x - viewport.x) / viewport.z;
    float y = -1 + 2 * (bound.y - viewport.y) / viewport.w;
    gl_Position = vec4(x, y, 0, 1);
    float w = bound.z - bound.x;
    float h = bound.w - bound.y;
    vs_out.w = 2 * w / viewport.z;
    vs_out.h = 2 * h / viewport.w;
    vs_out.uv_bound = uv_bound;
}
