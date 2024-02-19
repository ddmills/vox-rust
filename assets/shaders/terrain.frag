#version 450

layout(location = 0) in vec2 v_Uv;
layout(location = 1) in vec3 v_Normal;

layout(location = 0) out vec4 o_Target;

void main() {
    o_Target = vec4(1.0, 0, 1.0, 1.0);
}
