// #extension OES_texture_float: enable;

precision highp float;

uniform sampler2D tex;
varying vec2 vec_uv;

void main() {
    vec4 color = texture2D(tex, vec_uv);
    gl_FragColor = color;
    // gl_FragColor = vec4(vec_uv.x, 1.0, 1.0, 1.0);
    // gl_FragColor = vec4(1.0, 1.0, 1.0, 1.0);
}