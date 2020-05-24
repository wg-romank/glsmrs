precision highp float;

attribute vec2 position;
attribute vec2 uv;

uniform float time;

varying vec2 vec_uv;

void main() {
    float scale = time / 100.0;
    gl_Position = vec4(position, 0.0, scale);
    vec_uv = uv;
}