// Compute shader for Linearization (sRGB -> Linear)
// This is a simple test case for the GPU pipeline.

@group(0) @binding(0) var input_texture: texture_2d<f32>;
@group(0) @binding(1) var output_texture: texture_storage_2d<rgba32float, write>;

// sRGB to Linear approximation
fn srgb_to_linear(x: f32) -> f32 {
    if (x <= 0.04045) {
        return x / 12.92;
    } else {
        return pow((x + 0.055) / 1.055, 2.4);
    }
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let dimensions = textureDimensions(input_texture);
    let x = global_id.x;
    let y = global_id.y;

    if (x >= dimensions.x || y >= dimensions.y) {
        return;
    }

    let color = textureLoad(input_texture, vec2<i32>(i32(x), i32(y)), 0);
    
    // Convert RGB to Linear
    let r = srgb_to_linear(color.r);
    let g = srgb_to_linear(color.g);
    let b = srgb_to_linear(color.b);
    // Alpha remains unchanged
    let a = color.a;

    textureStore(output_texture, vec2<i32>(i32(x), i32(y)), vec4<f32>(r, g, b, a));
}
