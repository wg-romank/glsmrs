precision highp float;

uniform sampler2D tex;
varying vec2 vec_uv;

void main() {
    vec4 color = texture2D(tex, vec_uv);
    gl_FragColor = vec4(color.xyz, 1.0);
}