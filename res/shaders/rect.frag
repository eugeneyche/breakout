#version 330

in FS_IN {
    flat vec4 color;
} fs_in;

out vec4 fs_out;

void main() {
    fs_out = fs_in.color;
}
