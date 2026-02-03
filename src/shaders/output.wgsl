struct OutputUniforms {
    t_min: vec3<f32>,
    _pad1: f32,
    t_max: vec3<f32>,
    _pad2: f32,
    d_min: vec3<f32>,
    _pad3: f32,
    
    paper_gamma: f32,
    saturation: f32,
    output_mode: u32, // 0=Negative, 1=Positive
    
    width: u32,
    height: u32,
    _pad4: f32,
}

@group(0) @binding(0) var<storage, read> input_buffer: array<f32>;
@group(0) @binding(1) var<storage, read_write> output_buffer: array<f32>;
@group(0) @binding(2) var<uniform> uniforms: OutputUniforms;

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

fn density_to_transmission(d: f32) -> f32 {
    return pow(10.0, -d);
}

fn apply_dye_self_absorption(density: f32, transmission: f32) -> f32 {
    if (density > 1.5) {
        let correction = 1.0 + (density - 1.5) * 0.02;
        return transmission * clamp(correction, 0.97, 1.03);
    }
    return transmission;
}

fn linear_to_srgb(v: f32) -> f32 {
    if (v <= 0.0031308) {
        return 12.92 * v;
    } else {
        return 1.055 * pow(v, 1.0 / 2.4) - 0.055;
    }
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }
    
    let density = read_pixel(x, y);
    
    // Net Density
    let net_r = max(density.r - uniforms.d_min.r, 0.0);
    let net_g = max(density.g - uniforms.d_min.g, 0.0);
    let net_b = max(density.b - uniforms.d_min.b, 0.0);
    
    var t_r = density_to_transmission(net_r);
    var t_g = density_to_transmission(net_g);
    var t_b = density_to_transmission(net_b);
    
    // Apply Dye Self Absorption (common for both modes logic in CPU code?)
    // CPU: output_mode logic calls apply_dye_self_absorption in both branches.
    t_r = apply_dye_self_absorption(net_r, t_r);
    t_g = apply_dye_self_absorption(net_g, t_g);
    t_b = apply_dye_self_absorption(net_b, t_b);
    
    var r_lin = 0.0;
    var g_lin = 0.0;
    var b_lin = 0.0;
    
    if (uniforms.output_mode == 0u) { // Negative
        r_lin = clamp(t_r, 0.0, 1.0);
        g_lin = clamp(t_g, 0.0, 1.0);
        b_lin = clamp(t_b, 0.0, 1.0);
    } else { // Positive
        // Normalize
        let denom_r = max(uniforms.t_max.r - uniforms.t_min.r, 1e-6);
        let denom_g = max(uniforms.t_max.g - uniforms.t_min.g, 1e-6);
        let denom_b = max(uniforms.t_max.b - uniforms.t_min.b, 1e-6);
        
        let n_r = clamp(uniforms.t_max.r - t_r, 0.0, denom_r) / denom_r;
        let n_g = clamp(uniforms.t_max.g - t_g, 0.0, denom_g) / denom_g;
        let n_b = clamp(uniforms.t_max.b - t_b, 0.0, denom_b) / denom_b;
        
        r_lin = pow(n_r, uniforms.paper_gamma);
        g_lin = pow(n_g, uniforms.paper_gamma);
        b_lin = pow(n_b, uniforms.paper_gamma);
    }
    
    // Saturation
    if (uniforms.saturation != 1.0) {
        let lum = 0.2126 * r_lin + 0.7152 * g_lin + 0.0722 * b_lin;
        r_lin = lum + (r_lin - lum) * uniforms.saturation;
        g_lin = lum + (g_lin - lum) * uniforms.saturation;
        b_lin = lum + (b_lin - lum) * uniforms.saturation;
    }
    
    // Convert to sRGB
    let r_out = linear_to_srgb(clamp(r_lin, 0.0, 1.0));
    let g_out = linear_to_srgb(clamp(g_lin, 0.0, 1.0));
    let b_out = linear_to_srgb(clamp(b_lin, 0.0, 1.0));
    
    write_pixel(x, y, vec3<f32>(r_out, g_out, b_out));
}
