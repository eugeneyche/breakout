#version 330

layout(points) in;

in GS_IN {
    flat float w;
    flat float h;
    flat vec4 uv_bound;
    flat vec4 color;
} gs_in [];

layout(triangle_strip, max_vertices = 4) out;

out FS_IN {
    smooth vec2 uv;
    flat vec4 color;
} gs_out;

void main() {
    gs_out.color = gs_in[0].color;

    gl_Position = gl_in[0].gl_Position;
    gs_out.uv = gs_in[0].uv_bound.xy;
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(0, gs_in[0].h, 0, 0);
    gs_out.uv = gs_in[0].uv_bound.xw;
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(gs_in[0].w, 0, 0, 0);
    gs_out.uv = gs_in[0].uv_bound.zy;
    EmitVertex();

    gl_Position = gl_in[0].gl_Position + vec4(gs_in[0].w, gs_in[0].h, 0, 0);
    gs_out.uv = gs_in[0].uv_bound.zw;
    EmitVertex();
}
