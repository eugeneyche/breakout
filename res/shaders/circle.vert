#version 330
layout(location = 0) in vec2 center;
layout(location = 1) in float radius;
layout(location = 2) in vec4 color;


uniform vec4 viewport;

out GS_IN {
    flat vec2 dimensions;
    flat vec4 color;
} vs_out;

void main() {
    vec2 pos = vec2(-1, -1) + 2 * (center - vec2(radius, radius) - viewport.xy) / viewport.zw;
    vec2 dimensions = 4 * vec2(radius, radius) / viewport.zw;
    gl_Position = vec4(pos, 0, 1);
    vs_out.dimensions = dimensions;
    vs_out.color = color;
}

