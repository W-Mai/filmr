struct Uniforms {
    width: u32,
    height: u32,
    threshold: f32,
    _pad1: f32,
    sigma: f32,
    strength: f32,
    tint_r: f32,
    tint_g: f32,
    tint_b: f32,
    _pad2: f32,
    _pad3: f32,
    _pad4: f32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

// Helper to read a pixel (Linear RGB)
fn read_pixel(x: u32, y: u32) -> vec3<f32> {
    let clamp_x = min(max(x, 0u), uniforms.width - 1u);
    let clamp_y = min(max(y, 0u), uniforms.height - 1u);
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

fn luminance(c: vec3<f32>) -> f32 {
    return 0.2126 * c.r + 0.7152 * c.g + 0.0722 * c.b;
}

fn gaussian(x: f32, sigma: f32) -> f32 {
    return exp(-(x * x) / (2.0 * sigma * sigma));
}

// Pass 1: Threshold + Horizontal Blur
@compute @workgroup_size(16, 16, 1)
fn main_x(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }

    var sum = vec3<f32>(0.0);
    var weight_sum = 0.0;
    
    // Dynamic kernel radius based on sigma (3*sigma rule)
    // Clamp max radius to avoid TDR/timeout
    let radius = min(i32(ceil(3.0 * uniforms.sigma)), 50); 

    for (var i = -radius; i <= radius; i++) {
        let sample_x = i32(x) + i;
        let pixel = read_pixel(u32(sample_x), y);
        
        // Apply thresholding on the fly during the first pass read
        let lum = luminance(pixel);
        var thresholded = pixel;
        if (lum < uniforms.threshold) {
            thresholded = vec3<f32>(0.0);
        } else {
            thresholded = max(pixel - vec3<f32>(uniforms.threshold), vec3<f32>(0.0));
        }

        let w = gaussian(f32(i), uniforms.sigma);
        sum += thresholded * w;
        weight_sum += w;
    }

    if (weight_sum > 0.0) {
        sum = sum / weight_sum;
    }

    write_pixel(x, y, sum);
}

// Pass 2: Vertical Blur + Blend
// Note: output_buffer in this pass is the FINAL image. 
// input_buffer is the result of Pass 1 (Horizontal Blur).
// BUT we also need the ORIGINAL image to blend!
// Currently we only have bindings for Input and Output.
// We need a 3rd binding for Original Input if we want to blend here.
// Or we output the blurred map, and do a 3rd "Blend" pass.
// Let's add a 3rd binding: @binding(3) var<storage, read> original_buffer: array<f32>;

@group(0) @binding(3) var<storage, read> original_buffer: array<f32>;

fn read_original_pixel(x: u32, y: u32) -> vec3<f32> {
    let idx = (y * uniforms.width + x) * 3u;
    return vec3<f32>(original_buffer[idx], original_buffer[idx+1u], original_buffer[idx+2u]);
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
        // Read from Input (which is Pass 1 output = Horizontally blurred)
        let pixel = read_pixel(x, u32(sample_y));
        
        let w = gaussian(f32(i), uniforms.sigma);
        sum += pixel * w;
        weight_sum += w;
    }

    if (weight_sum > 0.0) {
        sum = sum / weight_sum;
    }

    // Now Blend with Original
    let original = read_original_pixel(x, y);
    let halation = sum;
    
    let tint = vec3<f32>(uniforms.tint_r, uniforms.tint_g, uniforms.tint_b);
    let result = original + halation * tint * uniforms.strength;

    write_pixel(x, y, result);
}
