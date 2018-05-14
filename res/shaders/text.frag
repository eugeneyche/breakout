#version 330

in FS_IN {
    smooth vec2 uv;
    flat vec4 color;
} fs_in;

uniform sampler2D font_tex;

out vec4 fs_out;

void main() {
    fs_out = vec4(fs_in.color.xyz, fs_in.color.w * texture(font_tex, fs_in.uv).r);
}
