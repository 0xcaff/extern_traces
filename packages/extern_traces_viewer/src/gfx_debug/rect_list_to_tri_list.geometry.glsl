#version 410 core

layout(triangles) in;
layout(triangle_strip, max_vertices = 6) out;

layout(location = 0) in vec4 inAttribute[];
layout(location = 0) out vec4 outAttribute;

vec3 projectVec(vec3 a, vec3 b) {
    float dotProduct = dot(a, b);
    float magnitudeSquared = dot(b, b);
    return (dotProduct / magnitudeSquared) * b;
}

void main() {
    {
        gl_Position = gl_in[0].gl_Position;
        outAttribute = inAttribute[0];
        EmitVertex();

        gl_Position = gl_in[1].gl_Position;
        outAttribute = inAttribute[1];
        EmitVertex();

        gl_Position = gl_in[2].gl_Position;
        outAttribute = inAttribute[2];
        EmitVertex();
        EndPrimitive();
    }

    {
        gl_Position = gl_in[1].gl_Position;
        outAttribute = inAttribute[1];
        EmitVertex();

        gl_Position = gl_in[2].gl_Position;
        outAttribute = inAttribute[2];
        EmitVertex();

        vec4 u = gl_in[1].gl_Position - gl_in[0].gl_Position;
        vec4 v = gl_in[2].gl_Position - gl_in[0].gl_Position;

        vec3 n = cross(u.xyz, v.xyz);
        vec3 p = cross(n, u.xyz);

        gl_Position = vec4(gl_in[1].gl_Position.xyz + projectVec(v.xyz, p), 1);

        // todo: correctly remap this one as well
        outAttribute = inAttribute[0];
        EmitVertex();

        EndPrimitive();

    }
}
