#version 150 core

uniform sampler2D albedo;

in VertexData {
  vec4 position;
  vec2 tex_coord;
} vertex;

out vec4 color;

void main(){
    color = vec4(1., 1., 1., 1.);
}
