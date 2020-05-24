attribute vec2 position;
attribute vec2 uv;

varying vec2 vec_uv;

void main() {
    gl_Position = vec4(position, 0.0, 1.0);
    vec_uv = uv;
}