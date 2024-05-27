// Vertex shader

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) color: vec3<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec3<f32>,
};

struct CameraUniform {
    view_proj: mat4x4<f32>,
};

@group(0) @binding(0)
var<uniform> camera: CameraUniform;


@vertex
fn vs_main(
    model: VertexInput,
) -> VertexOutput {
    var out: VertexOutput;
    let x = mat3x3(camera.view_proj[0].xyz, camera.view_proj[1].xyz, camera.view_proj[2].xyz);

    let xx = x[0].x * model.position.x + x[1].x * model.position.y + x[2].x;
    let y = x[0].y * model.position.x + x[1].y * model.position.y + x[2].y;

    out.color = model.color;
    out.clip_position = vec4<f32>(xx, y, 1.0, 1.0);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return vec4<f32>(in.color, 1.0);
}

@fragment
fn curve_fs_main(in: VertexOutput) {
    if in.color.x * in.color.x - in.color.y > 0 {
        discard;
    }
}