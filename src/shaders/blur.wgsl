struct Uniforms {
    width: u32,
    height: u32,
    sigma: f32,
    _pad: f32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

fn read_pixel(x: i32, y: i32) -> vec3<f32> {
    let clamp_x = u32(clamp(x, 0, i32(uniforms.width) - 1));
    let clamp_y = u32(clamp(y, 0, i32(uniforms.height) - 1));
    let idx = (clamp_y * uniforms.width + clamp_x) * 3u;
    return vec3<f32>(input_buffer[idx], input_buffer[idx+1u], input_buffer[idx+2u]);
}

fn write_pixel(x: u32, y: u32, color: vec3<f32>) {
    if (x >= uniforms.width || y >= uniforms.height) { return; }
    let idx = (y * uniforms.width + x) * 3u;
    output_buffer[idx] = color.r;
    output_buffer[idx+1u] = color.g;
    output_buffer[idx+2u] = color.b;
}

fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma));
}

@compute @workgroup_size(16, 16, 1)
fn main_x(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }

    var sum = vec3<f32>(0.0);
    var weight_sum = 0.0;
    
    let radius = min(i32(ceil(3.0 * uniforms.sigma)), 50); 

    for (var i = -radius; i <= radius; i++) {
        let sample_x = i32(x) + i;
        // Read from input_buffer
        let pixel = read_pixel(sample_x, i32(y));
        let w = gaussian(f32(i), uniforms.sigma);
        sum += pixel * w;
        weight_sum += w;
    }

    if (weight_sum > 0.0) {
        sum = sum / weight_sum;
    }

    write_pixel(x, y, sum);
}

@compute @workgroup_size(16, 16, 1)
fn main_y(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }

    var sum = vec3<f32>(0.0);
    var weight_sum = 0.0;
    
    let radius = min(i32(ceil(3.0 * uniforms.sigma)), 50); 

    for (var i = -radius; i <= radius; i++) {
        let sample_y = i32(y) + i;
        // Read from input_buffer (which is temp result from Pass X)
        let pixel = read_pixel(i32(x), sample_y);
        let w = gaussian(f32(i), uniforms.sigma);
        sum += pixel * w;
        weight_sum += w;
    }

    if (weight_sum > 0.0) {
        sum = sum / weight_sum;
    }

    write_pixel(x, y, sum);
}
