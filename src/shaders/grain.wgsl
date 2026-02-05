struct GrainUniforms {
    width: u32,
    height: u32,
    seed: f32,
    alpha: f32,
    sigma_read: f32,
    roughness: f32,
    monochrome: u32, // 0 or 1
    color_correlation: f32,
    shadow_noise: f32,
    highlight_coarseness: f32,
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

fn value_noise_gaussian(uv: vec2<f32>) -> f32 {
    let i = floor(uv);
    let f = fract(uv);
    let u = f * f * (3.0 - 2.0 * f);

    let ga = box_muller(hash2(i + vec2<f32>(0.0, 0.0)));
    let gb = box_muller(hash2(i + vec2<f32>(1.0, 0.0)));
    let gc = box_muller(hash2(i + vec2<f32>(0.0, 1.0)));
    let gd = box_muller(hash2(i + vec2<f32>(1.0, 1.0)));

    return mix(mix(ga, gb, u.x), mix(gc, gd, u.x), u.y);
}

fn sample_noise(d: f32, uv: vec2<f32>, scale: f32) -> f32 {
    // Section 7: Grain Statistics Model.
    // Var(D) = alpha * D^1.5 + sigma_read^2 + shadow_noise / (D + 0.1)
    let d_clamped = max(d, 0.0);
    
    // Photon Shot Noise (Shadows)
    let shot_variance = clamp(uniforms.shadow_noise * (1.0 / (d_clamped + 0.1)), 0.0, 10.0);
    
    let base_variance = uniforms.alpha * pow(d_clamped, 1.5) + pow(uniforms.sigma_read, 2.0) + shot_variance;
    
    // Roughness modulation:
    let pi = 3.14159265;
    let modulation = 1.0 + uniforms.roughness * sin(pi * clamp(d_clamped, 0.0, 1.0));
    
    let variance = base_variance * modulation;
    let std_dev = sqrt(max(variance, 0.0));
    
    if (std_dev <= 0.0) {
        return 0.0;
    }
    
    if (scale <= 1.001) {
        let h = hash2(uv);
        return box_muller(h) * std_dev;
    } else {
        return value_noise_gaussian(uv / scale) * std_dev;
    }
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;
    
    if (x >= uniforms.width || y >= uniforms.height) { return; }
    
    let density = read_pixel(x, y);
    let uv = vec2<f32>(f32(x), f32(y)) + vec2<f32>(uniforms.seed * 100.0);
    
    var noise = vec3<f32>(0.0);
    
    // Highlight Coarseness Factor
    // Scales with density (green channel proxy), D^2 behavior
    // Normalized D (approx Dmax=2.5)
    let d_luma_proxy = density.g;
    let clump_intensity = pow(clamp(d_luma_proxy / 2.5, 0.0, 1.0), 2.0) * uniforms.highlight_coarseness;
    
    if (uniforms.monochrome == 1u) {
        // Monochrome uses Green channel density (or luminance)
        let d = density.g; 
        let n_fine = sample_noise(d, uv, 1.0);
        var n_total = n_fine;
        
        if (uniforms.highlight_coarseness > 0.0) {
            let n_coarse = sample_noise(d, uv, 3.0);
            n_total += n_coarse * clump_intensity;
        }
        
        noise = vec3<f32>(n_total);
    } else {
        // "Natural" grain simulation:
        // 1. Generate Luma Noise based on weighted luminance density
        // 2. Generate Independent Chroma Noise
        // 3. Mix based on color_correlation factor

        // Calculate Luminance Density (approximate)
        let d_lum = 0.2126 * density.r + 0.7152 * density.g + 0.0722 * density.b;
        
        // Master Luma Noise
        let n_shared_fine = sample_noise(d_lum, uv, 1.0);
        var n_shared = n_shared_fine;
        if (uniforms.highlight_coarseness > 0.0) {
             let n_shared_coarse = sample_noise(d_lum, uv, 3.0);
             n_shared += n_shared_coarse * clump_intensity;
        }

        // Independent Channel Noise
        let uv_r = uv;
        let uv_g = uv + vec2<f32>(12.34, 56.78);
        let uv_b = uv + vec2<f32>(90.12, 34.56);
        
        // Red
        let n_r_fine = sample_noise(density.r, uv_r, 1.0);
        var n_r = n_r_fine;
        if (uniforms.highlight_coarseness > 0.0) {
            let n_r_coarse = sample_noise(density.r, uv_r, 3.0);
            n_r += n_r_coarse * clump_intensity;
        }
        
        // Green
        let n_g_fine = sample_noise(density.g, uv_g, 1.0);
        var n_g = n_g_fine;
        if (uniforms.highlight_coarseness > 0.0) {
            let n_g_coarse = sample_noise(density.g, uv_g, 3.0);
            n_g += n_g_coarse * clump_intensity;
        }
        
        // Blue
        let n_b_fine = sample_noise(density.b, uv_b, 1.0);
        var n_b = n_b_fine;
        if (uniforms.highlight_coarseness > 0.0) {
            let n_b_coarse = sample_noise(density.b, uv_b, 3.0);
            n_b += n_b_coarse * clump_intensity;
        }
        
        let corr = uniforms.color_correlation;
        
        // Mix: Result = Correlation * Shared + (1 - Correlation) * Independent
        noise.r = corr * n_shared + (1.0 - corr) * n_r;
        noise.g = corr * n_shared + (1.0 - corr) * n_g;
        noise.b = corr * n_shared + (1.0 - corr) * n_b;
    }
    
    // Additive noise to density, clamped to 0
    let out_density = max(density + noise, vec3<f32>(0.0));
    
    write_pixel(x, y, out_density);
}
