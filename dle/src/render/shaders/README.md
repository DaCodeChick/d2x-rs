# Shaders

This directory contains GLSL shaders for the DLE renderer using Qt RHI (Render Hardware Interface).

## Files

- `*.vert` - Vertex shader source (GLSL) - **committed to git**
- `*.frag` - Fragment shader source (GLSL) - **committed to git**
- `*.qsb` - Compiled shader bundles - **generated during build, not committed**

## Workflow

### Normal Development

Just edit the GLSL source files. CMake automatically compiles them to `.qsb` during build:

```bash
# Edit a shader
vim basic.vert

# Build - shader is automatically compiled to basic.vert.qsb
cd dle/build
cmake --build .
```

### Manual Compilation (if needed)

```bash
qsb --glsl "100es,120,150" --hlsl 50 --msl 12 -o basic.vert.qsb basic.vert
```

## Shader Variables

### Vertex Shader Inputs
- `layout(location = 0) in vec3 position` - Vertex position
- `layout(location = 1) in vec4 color` - Vertex color

### Uniforms
- `layout(binding = 0) uniform buf { mat4 mvp; }` - Model-View-Projection matrix

### Outputs
- `out vec4 v_color` - Color passed to fragment shader
