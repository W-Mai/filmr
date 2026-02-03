struct LightLeak {
    position: vec2<f32>,
    radius: f32,
    intensity: f32,
    color: vec3<f32>,
    shape: u32,
    rotation: f32,
    roughness: f32,
    padding: vec2<f32>, // Pad to 16-byte alignment if needed, or just total size
}

struct Uniforms {
    width: u32,
    height: u32,
    leak_count: u32,
    _pad: u32,
}

@group(0) @binding(0)
var<storage, read_write> image_buffer: array<f32>;

@group(0) @binding(1)
var<storage, read> leaks_buffer: array<LightLeak>;

@group(0) @binding(2)
var<uniform> uniforms: Uniforms;

fn pseudo_noise(coord: vec2<f32>) -> f32 {
    return fract(sin(dot(coord, vec2<f32>(12.9898, 78.233))) * 43758.5453);
}

@compute @workgroup_size(16, 16, 1)
fn main(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let x = global_id.x;
    let y = global_id.y;

    if (x >= uniforms.width || y >= uniforms.height) {
        return;
    }

    let index = (y * uniforms.width + x) * 3u;
    
    // Read current pixel (Linear RGB)
    var r = image_buffer[index];
    var g = image_buffer[index + 1u];
    var b = image_buffer[index + 2u];

    let width_f = f32(uniforms.width);
    let height_f = f32(uniforms.height);
    let min_dim = min(width_f, height_f);
    let x_f = f32(x);
    let y_f = f32(y);

    for (var i = 0u; i < uniforms.leak_count; i = i + 1u) {
        let leak = leaks_buffer[i];
        
        let center_x = leak.position.x * width_f;
        let center_y = leak.position.y * height_f;
        let radius_px = leak.radius * min_dim;
        let radius_sq = radius_px * radius_px;

        let dx = x_f - center_x;
        let dy = y_f - center_y;
        let dist_sq = dx * dx + dy * dy;

        // Optimization: Rough bounding check is hard inside loop without branching, 
        // but simple distance check is cheap.
        if (dist_sq < radius_sq) {
            let dist = sqrt(dist_sq);
            var falloff = 0.0;

            // Shape: 0=Circle, 1=Linear, 2=Organic, 3=Plasma
            if (leak.shape == 0u) { // Circle
                let t = dist / radius_px;
                falloff = pow(max(0.0, 1.0 - t), 2.0);
            } else if (leak.shape == 1u) { // Linear
                let nx = -sin(leak.rotation);
                let ny = cos(leak.rotation);
                let dist_normal = abs(dx * nx + dy * ny);
                let t = dist_normal / radius_px;
                falloff = pow(max(0.0, 1.0 - t), 2.0);
            } else if (leak.shape == 2u) { // Organic
                let noise_scale = 0.05;
                let n = pseudo_noise(vec2<f32>(x_f * noise_scale, y_f * noise_scale));
                let distorted_radius = radius_px * (1.0 - leak.roughness * 0.5 + n * leak.roughness);
                let t = dist / distorted_radius;
                falloff = pow(max(0.0, 1.0 - t), 3.0);
            } else if (leak.shape == 3u) { // Plasma
                let freq = 0.1 / (leak.radius + 0.01);
                let phase = leak.rotation * 5.0;
                let v = (sin(x_f * freq + phase) + cos(y_f * freq + phase)) * 0.5 + 0.5;
                let t = dist / radius_px;
                let base_falloff = pow(max(0.0, 1.0 - t), 2.0);
                falloff = base_falloff * (1.0 - leak.roughness + v * leak.roughness);
            }

            let factor = falloff * leak.intensity;
            r = r + leak.color.x * factor;
            g = g + leak.color.y * factor;
            b = b + leak.color.z * factor;
        }
    }

    // Write back
    image_buffer[index] = r;
    image_buffer[index + 1u] = g;
    image_buffer[index + 2u] = b;
}
