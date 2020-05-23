precision highp float;

varying vec2 vec_uv;

void main() {
    gl_FragColor = vec4(vec_uv.x, 1.0, 1.0, 1.0);
    // gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}