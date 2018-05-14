#version 330
layout(location = 0) in vec2 position;
layout(location = 1) in vec4 color;

uniform vec4 viewport;

out FS_IN {
    smooth vec4 color;
} vs_out;

void main() {
    vec2 pos = vec2(-1, -1) + 2 * (position - viewport.xy) / viewport.zw;
    gl_Position = vec4(pos, 0, 1);
    vs_out.color = color;
}
