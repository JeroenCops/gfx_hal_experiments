#version 450

layout(location = 0) in vec3 a_pos;

layout (push_constant) uniform PushConsts {
    float uPointSize;
} pushConsts;

void main() {
  gl_PointSize = 50.0;
  gl_Position = vec4(a_pos, 1.0);
}
