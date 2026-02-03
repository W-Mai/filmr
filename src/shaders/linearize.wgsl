@group(0) @binding(0) var<storage, read> input_buffer: array<u32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;

struct Uniforms {
    width: u32,
    height: u32,
}
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

fn srgb_to_linear(x: f32) -> f32 {
    if (x <= 0.04045) {
        return x / 12.92;
    } else {
        return pow((x + 0.055) / 1.055, 2.4);
    }
}

fn read_u8(byte_index: u32) -> f32 {
    let word_index = byte_index / 4u;
    let byte_offset = (byte_index % 4u) * 8u;
    let word = input_buffer[word_index];
    let byte_val = (word >> byte_offset) & 0xFFu;
    return f32(byte_val) / 255.0;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    if (x >= uniforms.width || y >= uniforms.height) {
        return;
    }

    let pixel_index = y * uniforms.width + x;
    let base_byte_index = pixel_index * 3u;

    // Read RGB
    let r = srgb_to_linear(read_u8(base_byte_index));
    let g = srgb_to_linear(read_u8(base_byte_index + 1u));
    let b = srgb_to_linear(read_u8(base_byte_index + 2u));

    // Write packed RGB f32
    let base_float_index = pixel_index * 3u;
    output_buffer[base_float_index] = r;
    output_buffer[base_float_index + 1u] = g;
    output_buffer[base_float_index + 2u] = b;
}
