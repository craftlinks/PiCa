struct Uniforms {
    mvpMatrix : mat4x4<f32>;
};
[[binding(0), group(0)]]
var<uniform> uniforms: Uniforms;


struct Camera {
    view_pos: vec4<f32>;
    view_proj: mat4x4<f32>;
};
[[group(1), binding(0)]]
var<uniform> camera: Camera;

struct Output {
    [[builtin(position)]] Position : vec4<f32>;
    [[location(0)]] vColor : vec4<f32>;
};

struct InstanceInput {
    [[location(5)]] model_matrix_0: vec4<f32>;
    [[location(6)]] model_matrix_1: vec4<f32>;
    [[location(7)]] model_matrix_2: vec4<f32>;
    [[location(8)]] model_matrix_3: vec4<f32>;
};

[[stage(vertex)]]
fn vs_main([[location(0)]] pos: vec4<f32>, [[location(1)]] color: vec4<f32>, instance: InstanceInput) -> Output {
    var output: Output;
    
    let model_matrix = mat4x4<f32>(
        instance.model_matrix_0,
        instance.model_matrix_1,
        instance.model_matrix_2,
        instance.model_matrix_3,
    );
    
    // output.Position = uniforms.mvpMatrix * model_matrix * pos;
    output.Position = camera.view_proj * uniforms.mvpMatrix * model_matrix * pos;
    output.vColor = color;
    return output;
}

[[stage(fragment)]]
fn fs_main([[location(0)]] vColor: vec4<f32>) -> [[location(0)]] vec4<f32> {
    return vColor;
}