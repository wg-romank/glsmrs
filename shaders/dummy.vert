attribute vec4 position;

void main() {
    // gl_Position = vec4(position, 0.0, 0.0);
    gl_Position = position;
}