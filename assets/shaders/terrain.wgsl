#import bevy_pbr::mesh_functions::{get_model_matrix, mesh_position_local_to_clip}

struct TerrainMaterial {
    color: vec4<f32>,
}

@group(2) @binding(0) var<uniform> material: TerrainMaterial;

struct Vertex {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(1) texture_idx: u32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) texture_idx: u32,
};

@vertex 
fn vertex(vertex: Vertex) -> VertexOutput {
    var out: VertexOutput;
    out.clip_position = mesh_position_local_to_clip(
        get_model_matrix(vertex.instance_index),
        vec4<f32>(vertex.position, 1.0),
    );
    out.texture_idx = vertex.texture_idx;
    return out;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    var dirt: u32 = 1u;

    if mesh.texture_idx == dirt {
        return vec4<f32>(1.0, 0.0, 0.0, 1.0); // Return red color
    }

    return vec4<f32>(0.0, 0.0, 1.0, 1.0); // Return red color
}
