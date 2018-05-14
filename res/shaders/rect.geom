#version 330
layout(points) in;

in GS_IN {
    flat vec2 dimensions;
    flat vec4 color;
} gs_in [];

layout(triangle_strip, max_vertices = 4) out;

out FS_IN {
    flat vec4 color;
} gs_out;

void main() {
    gs_out.color = gs_in[0].color;

    gl_Position = gl_in[0].gl_Position;
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(0, gs_in[0].dimensions.y, 0, 0);
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(gs_in[0].dimensions.x, 0, 0, 0);
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(
        gs_in[0].dimensions.x,
        gs_in[0].dimensions.y,
        0,
        0
    );
    EmitVertex();
}
