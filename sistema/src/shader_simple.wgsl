// shader_simple.wgsl - Shader simplificado para debug

struct Uniforms {
    time: f32,
    shader_type: u32,
    resolution: vec2<f32>,
}

struct ModelTransform {
    model_matrix: mat4x4<f32>,
    view_proj_matrix: mat4x4<f32>,
}

@group(0) @binding(0)
var<uniform> uniforms: Uniforms;

@group(1) @binding(0)
var<uniform> transform: ModelTransform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) world_pos: vec3<f32>,
    @location(1) normal: vec3<f32>,
}

@vertex
fn vs_main(input: VertexInput) -> VertexOutput {
    var output: VertexOutput;
    
    // Transformar posición
    let world_pos = transform.model_matrix * vec4<f32>(input.position, 1.0);
    output.world_pos = world_pos.xyz;
    
    // Transformar normal
    let normal_matrix = mat3x3<f32>(
        transform.model_matrix[0].xyz,
        transform.model_matrix[1].xyz,
        transform.model_matrix[2].xyz
    );
    output.normal = normalize(normal_matrix * input.normal);
    
    // Aplicar proyección
    output.clip_position = transform.view_proj_matrix * world_pos;
    
    return output;
}

@fragment
fn fs_main(input: VertexOutput) -> @location(0) vec4<f32> {
    // Colores simples según shader_type
    var color: vec3<f32>;
    
    if (uniforms.shader_type == 1u) {
        // Sol - amarillo
        color = vec3<f32>(1.0, 0.9, 0.2);
    } else if (uniforms.shader_type == 2u) {
        // Planeta rocoso - naranja
        color = vec3<f32>(0.8, 0.4, 0.2);
    } else if (uniforms.shader_type == 3u) {
        // Gigante gaseoso - azul claro
        color = vec3<f32>(0.3, 0.6, 0.9);
    } else if (uniforms.shader_type == 4u) {
        // Planeta con anillos - amarillo/marrón
        color = vec3<f32>(0.9, 0.7, 0.3);
    } else if (uniforms.shader_type == 5u) {
        // Volcánico - rojo
        color = vec3<f32>(0.9, 0.2, 0.1);
    } else {
        // Luna - gris
        color = vec3<f32>(0.7, 0.7, 0.7);
    }
    
    // Iluminación simple
    let light_dir = normalize(vec3<f32>(1.0, 1.0, 1.0));
    let diffuse = max(dot(normalize(input.normal), light_dir), 0.2);
    
    return vec4<f32>(color * diffuse, 1.0);
}
