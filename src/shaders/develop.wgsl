struct Curve {
    d_min: f32,
    d_max: f32,
    gamma: f32,
    exposure_offset: f32,
}

struct DevelopUniforms {
    // 3x3 Matrix columns (WGSL matrices are column-major)
    // We'll use manual vec3s for clarity
    matrix_r: vec3<f32>, // Row 0: m00, m01, m02
    matrix_g: vec3<f32>, // Row 1: m10, m11, m12
    matrix_b: vec3<f32>, // Row 2: m20, m21, m22
    
    curve_r: Curve,
    curve_g: Curve,
    curve_b: Curve,
    
    wb_r: f32,
    wb_g: f32,
    wb_b: f32,
    
    t_eff: f32,
    
    width: u32,
    height: u32,
    _pad: u32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;
@group(0) @binding(2) var<uniform> uniforms: DevelopUniforms;

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

// Approximation of erf(x)
fn erf(x: f32) -> f32 {
    let sign_x = sign(x);
    let a = abs(x);
    let t = 1.0 / (1.0 + 0.3275911 * a);
    let y = 1.0 - (((((1.061405429 * t - 1.453152027) * t) + 1.421413741) * t - 0.284496736) * t + 0.254829592) * t * exp(-a * a);
    return sign_x * y;
}

fn map_curve(log_e: f32, curve: Curve) -> f32 {
    let log_e0 = log(curve.exposure_offset) / log(10.0); // log10(offset)
    let range = curve.d_max - curve.d_min;
    
    if (range <= 0.0) {
        return curve.d_min;
    }
    
    let sqrt_pi = 1.7724539;
    let sigma = range / (curve.gamma * sqrt_pi);
    let z = (log_e - log_e0) / sigma;
    
    let val = 0.5 * (1.0 + erf(z));
    return curve.d_min + range * val;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }
    
    let linear_rgb = read_pixel(x, y);
    
    // 1. Apply Spectral Matrix
    // dot product of rows with input vector
    let r_exp_val = dot(uniforms.matrix_r, linear_rgb);
    let g_exp_val = dot(uniforms.matrix_g, linear_rgb);
    let b_exp_val = dot(uniforms.matrix_b, linear_rgb);
    
    // 2. Apply White Balance
    let r_balanced = r_exp_val * uniforms.wb_r;
    let g_balanced = g_exp_val * uniforms.wb_g;
    let b_balanced = b_exp_val * uniforms.wb_b;
    
    // 3. Apply Exposure Time (Reciprocity pre-calculated in t_eff)
    let r_exposure = r_balanced * uniforms.t_eff;
    let g_exposure = g_balanced * uniforms.t_eff;
    let b_exposure = b_balanced * uniforms.t_eff;
    
    // 4. Log10 and Map Curves
    let epsilon = 1e-6;
    let log_r = log(max(r_exposure, epsilon)) / log(10.0);
    let log_g = log(max(g_exposure, epsilon)) / log(10.0);
    let log_b = log(max(b_exposure, epsilon)) / log(10.0);
    
    let d_r = map_curve(log_r, uniforms.curve_r);
    let d_g = map_curve(log_g, uniforms.curve_g);
    let d_b = map_curve(log_b, uniforms.curve_b);
    
    write_pixel(x, y, vec3<f32>(d_r, d_g, d_b));
}
