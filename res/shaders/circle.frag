#version 330

in FS_IN {
    smooth vec2 xy;
    flat vec4 color;
} fs_in;

out vec4 fs_out;

void main() {
    if (dot(fs_in.xy, fs_in.xy) > 1) {
        discard;
    }

    fs_out = fs_in.color;
}
