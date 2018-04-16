#version 150 core

in vec3 in_position;
in vec2 tex_coord;
out vec2 tex_coord_out;

out VertexData {
  vec4 position;
  vec2 tex_coord;
} vertex;

void main() {
    vertex.position = vec4(in_position.xy - vec2(1., 1.) * vec2(2., 2.), 0, 1);
    vertex.tex_coord = tex_coord;
    gl_Position = vertex.position;
}
