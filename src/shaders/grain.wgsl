struct GrainUniforms {
    width: u32,
    height: u32,
    seed: f32,
    alpha: f32,
    sigma_read: f32,
    roughness: f32,
    monochrome: u32, // 0 or 1
    _pad: f32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;
@group(0) @binding(2) var<uniform> uniforms: GrainUniforms;

fn read_pixel(x: u32, y: u32) -> vec3<f32> {
    if (x >= uniforms.width || y >= uniforms.height) { return vec3<f32>(0.0); }
    let idx = (y * uniforms.width + x) * 3u;
    return vec3<f32>(input_buffer[idx], input_buffer[idx+1u], input_buffer[idx+2u]);
}

fn write_pixel(x: u32, y: u32, color: vec3<f32>) {
    if (x >= uniforms.width || y >= uniforms.height) { return; }
    let idx = (y * uniforms.width + x) * 3u;
    output_buffer[idx] = color.r;
    output_buffer[idx+1u] = color.g;
    output_buffer[idx+2u] = color.b;
}

// Hash function (Gold Noise derived or similar)
fn hash2(p: vec2<f32>) -> vec2<f32> {
    var p3 = fract(vec3<f32>(p.xyx) * vec3<f32>(0.1031, 0.1030, 0.0973));
    p3 = p3 + dot(p3, p3.yzx + 33.33);
    return fract((p3.xx + p3.yz) * p3.zy);
}

fn box_muller(uv: vec2<f32>) -> f32 {
    let u1 = max(uv.x, 1e-6); // Avoid log(0)
    let u2 = uv.y;
    return sqrt(-2.0 * log(u1)) * cos(6.2831853 * u2);
}

fn sample_noise(d: f32, uv: vec2<f32>) -> f32 {
    // Section 7: Grain Statistics Model.
    // Var(D) = alpha * D^1.5 + sigma_read^2
    let d_clamped = max(d, 0.0);
    let base_variance = uniforms.alpha * pow(d_clamped, 1.5) + pow(uniforms.sigma_read, 2.0);
    
    // Roughness modulation:
    // Adjusted Variance = Base_Variance * (1.0 + roughness * sin(pi * d))
    let pi = 3.14159265;
    let modulation = 1.0 + uniforms.roughness * sin(pi * clamp(d_clamped, 0.0, 1.0));
    
    let variance = base_variance * modulation;
    let std_dev = sqrt(max(variance, 0.0));
    
    if (std_dev <= 0.0) {
        return 0.0;
    }
    
    let h = hash2(uv);
    return box_muller(h) * std_dev;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }
    
    let density = read_pixel(x, y);
    let uv = vec2<f32>(f32(x), f32(y)) + vec2<f32>(uniforms.seed * 100.0);
    
    var noise = vec3<f32>(0.0);
    
    if (uniforms.monochrome == 1u) {
        // Monochrome uses Green channel density (or luminance)
        let d = density.g; 
        let n = sample_noise(d, uv);
        noise = vec3<f32>(n);
    } else {
        // Need 3 independent noise values.
        let uv_r = uv;
        let uv_g = uv + vec2<f32>(12.34, 56.78);
        let uv_b = uv + vec2<f32>(90.12, 34.56);
        
        let n_r = sample_noise(density.r, uv_r);
        let n_g = sample_noise(density.g, uv_g);
        let n_b = sample_noise(density.b, uv_b);
        
        // Simple approximation of chroma scaling from CPU code:
        // n_lum = (n_r + n_g + n_b) / 3.0
        // pixel = n_lum + (n_ch - n_lum) * chroma_scale
        // where chroma_scale = 0.3 (hardcoded in CPU)
        
        let n_lum = (n_r + n_g + n_b) / 3.0;
        let chroma_scale = 0.3;
        
        noise.r = n_lum + (n_r - n_lum) * chroma_scale;
        noise.g = n_lum + (n_g - n_lum) * chroma_scale;
        noise.b = n_lum + (n_b - n_lum) * chroma_scale;
    }
    
    // Additive noise to density, clamped to 0
    let out_density = max(density + noise, vec3<f32>(0.0));
    
    write_pixel(x, y, out_density);
}
