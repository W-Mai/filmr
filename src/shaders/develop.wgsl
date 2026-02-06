struct Curve {
    d_min: f32,
    d_max: f32,
    gamma: f32,
    exposure_offset: f32,
    shoulder_point: f32,
    _pad0: f32,
    _pad1: f32,
    _pad2: f32,
}

struct DevelopUniforms {
    // Spectral matrix: maps Linear RGB -> Film Layer Exposure
    spectral_r: vec3<f32>,
    _pad_sr: f32,
    spectral_g: vec3<f32>,
    _pad_sg: f32,
    spectral_b: vec3<f32>,
    _pad_sb: f32,

    // Color coupling matrix: applied to net densities
    color_r: vec3<f32>,
    _pad_cr: f32,
    color_g: vec3<f32>,
    _pad_cg: f32,
    color_b: vec3<f32>,
    _pad_cb: f32,

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
    _pad2: u32,
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

// Logistic sigmoid H-D curve (consistent with CPU map_smooth)
fn map_curve(log_e: f32, curve: Curve) -> f32 {
    let log_e0 = log(curve.exposure_offset) / log(10.0);
    let x = log_e - log_e0;
    let range = curve.d_max - curve.d_min;

    if (range <= 0.0) {
        return curve.d_min;
    }

    let k = 4.0 * curve.gamma / range;
    let sigmoid = 1.0 / (1.0 + exp(-k * x));
    return curve.d_min + range * sigmoid;
}

// Space charge limit compression at high densities
fn shoulder_softening(density: f32, shoulder_point: f32) -> f32 {
    if (density > shoulder_point) {
        let excess = density - shoulder_point;
        return density - (excess * excess) / (shoulder_point + excess);
    }
    return density;
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    if (x >= uniforms.width || y >= uniforms.height) { return; }

    let linear_rgb = read_pixel(x, y);

    // 1. Spectral matrix: Linear RGB -> Film Layer Exposure
    let r_exp_val = dot(uniforms.spectral_r, linear_rgb);
    let g_exp_val = dot(uniforms.spectral_g, linear_rgb);
    let b_exp_val = dot(uniforms.spectral_b, linear_rgb);

    // 2. White Balance
    let r_balanced = r_exp_val * uniforms.wb_r;
    let g_balanced = g_exp_val * uniforms.wb_g;
    let b_balanced = b_exp_val * uniforms.wb_b;

    // 3. Exposure (reciprocity pre-calculated in t_eff)
    let r_exposure = max(r_balanced, 0.0) * uniforms.t_eff;
    let g_exposure = max(g_balanced, 0.0) * uniforms.t_eff;
    let b_exposure = max(b_balanced, 0.0) * uniforms.t_eff;

    // 4. Log10 -> H-D Curve -> Density
    let epsilon = 1e-6;
    let log_r = log(max(r_exposure, epsilon)) / log(10.0);
    let log_g = log(max(g_exposure, epsilon)) / log(10.0);
    let log_b = log(max(b_exposure, epsilon)) / log(10.0);

    var d_r = map_curve(log_r, uniforms.curve_r);
    var d_g = map_curve(log_g, uniforms.curve_g);
    var d_b = map_curve(log_b, uniforms.curve_b);

    // 5. Shoulder softening
    d_r = shoulder_softening(d_r, uniforms.curve_r.shoulder_point);
    d_g = shoulder_softening(d_g, uniforms.curve_g.shoulder_point);
    d_b = shoulder_softening(d_b, uniforms.curve_b.shoulder_point);

    // 6. Net density -> Color coupling matrix -> Final density
    let net_r = max(d_r - uniforms.curve_r.d_min, 0.0);
    let net_g = max(d_g - uniforms.curve_g.d_min, 0.0);
    let net_b = max(d_b - uniforms.curve_b.d_min, 0.0);
    let net = vec3<f32>(net_r, net_g, net_b);

    let out_r = dot(uniforms.color_r, net) + uniforms.curve_r.d_min;
    let out_g = dot(uniforms.color_g, net) + uniforms.curve_g.d_min;
    let out_b = dot(uniforms.color_b, net) + uniforms.curve_b.d_min;

    write_pixel(x, y, vec3<f32>(out_r, out_g, out_b));
}
