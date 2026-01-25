use image::RgbImage;
use palette::{Srgb, Lab, FromColor};
use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Debug, Clone)]
pub struct FilmMetrics {
    // Basic Stats
    pub mean_rgb: [f32; 3],
    pub std_rgb: [f32; 3],
    pub skewness_rgb: [f32; 3],
    pub kurtosis_rgb: [f32; 3],
    pub quantiles_rgb: [[u8; 4]; 3], // p10, p50, p90, p99
    pub clipping_ratio: [f32; 2], // [zero_ratio, saturated_ratio]
    pub entropy: f32,
    pub dynamic_range: f32,
    
    // Color Science
    pub lab_mean: [f32; 3],
    pub lab_std: [f32; 3],
    pub rg_ratio: f32,
    pub bg_ratio: f32,
    pub saturation_mean: f32,
    pub saturation_skew: f32,
    pub cct_tint: (f32, f32), // (CCT, Tint)
    pub delta_e: Option<f32>, // Needs reference
    
    // Frequency / Texture
    pub laplacian_variance: f32,
    pub psd_slope: f32, // 1/f^beta, returns beta
    pub lbp_hist: [f32; 10], // Simplified LBP
    pub glcm_stats: [f32; 4], // Contrast, Correlation, Energy, Homogeneity
    
    // Perceptual / Structure
    pub ssim: Option<f32>, // Needs reference
}

impl FilmMetrics {
    pub fn analyze(img: &RgbImage) -> Self {
        let count = (img.width() * img.height()) as f32;
        if count == 0.0 {
            return Self::empty();
        }

        let mut sum_rgb = [0.0; 3];
        let mut sq_sum_rgb = [0.0; 3];
        
        let mut sum_lab = [0.0; 3];
        let mut sq_sum_lab = [0.0; 3];
        
        let mut sum_sat = 0.0;
        let mut sq_sum_sat = 0.0;
        
        let mut hist = [0u32; 256]; // For entropy (luminance)
        
        // Pass 1: Basic Sums
        for p in img.pixels() {
            let r = p[0] as f32;
            let g = p[1] as f32;
            let b = p[2] as f32;
            
            // RGB Stats
            for c in 0..3 {
                let v = p[c] as f32;
                sum_rgb[c] += v;
                sq_sum_rgb[c] += v * v;
            }
            
            // Lab & Saturation
            let srgb = Srgb::new(r/255.0, g/255.0, b/255.0);
            let lab: Lab = Lab::from_color(srgb);
            
            sum_lab[0] += lab.l;
            sum_lab[1] += lab.a;
            sum_lab[2] += lab.b;
            
            sq_sum_lab[0] += lab.l * lab.l;
            sq_sum_lab[1] += lab.a * lab.a;
            sq_sum_lab[2] += lab.b * lab.b;
            
            // Saturation (Lab Chroma)
            let chroma = (lab.a.powi(2) + lab.b.powi(2)).sqrt();
            sum_sat += chroma;
            sq_sum_sat += chroma * chroma;
            
            // Entropy (Luminance)
            let lum = (0.2126 * r + 0.7152 * g + 0.0722 * b) as u8;
            hist[lum as usize] += 1;
        }
        
        // Calculate Means and Stds
        let mut mean_rgb = [0.0; 3];
        let mut std_rgb = [0.0; 3];
        
        for c in 0..3 {
            mean_rgb[c] = sum_rgb[c] / count;
            let var = (sq_sum_rgb[c] / count) - (mean_rgb[c] * mean_rgb[c]);
            std_rgb[c] = if var > 0.0 { var.sqrt() } else { 0.0 };
        }
        
        let lab_mean = [sum_lab[0]/count, sum_lab[1]/count, sum_lab[2]/count];
        let lab_std = [
            ((sq_sum_lab[0]/count) - (lab_mean[0]*lab_mean[0])).sqrt().max(0.0),
            ((sq_sum_lab[1]/count) - (lab_mean[1]*lab_mean[1])).sqrt().max(0.0),
            ((sq_sum_lab[2]/count) - (lab_mean[2]*lab_mean[2])).sqrt().max(0.0),
        ];
        
        let sat_mean = sum_sat / count;
        let sat_var = (sq_sum_sat / count) - (sat_mean * sat_mean);
        let sat_std = if sat_var > 0.0 { sat_var.sqrt() } else { 0.0 };
        
        // Pass 2: Higher Moments (Skew/Kurtosis)
        let mut cube_sum_rgb = [0.0; 3];
        let mut quad_sum_rgb = [0.0; 3];
        let mut cube_sum_sat = 0.0;
        
        for p in img.pixels() {
            for c in 0..3 {
                let v = p[c] as f32;
                let diff = v - mean_rgb[c];
                cube_sum_rgb[c] += diff.powi(3);
                quad_sum_rgb[c] += diff.powi(4);
            }
            
            let r = p[0] as f32;
            let g = p[1] as f32;
            let b = p[2] as f32;
            let srgb = Srgb::new(r/255.0, g/255.0, b/255.0);
            let lab: Lab = Lab::from_color(srgb);
            let chroma = (lab.a.powi(2) + lab.b.powi(2)).sqrt();
            
            let diff_sat = chroma - sat_mean;
            cube_sum_sat += diff_sat.powi(3);
        }
        
        let mut skewness_rgb = [0.0; 3];
        let mut kurtosis_rgb = [0.0; 3];
        
        for c in 0..3 {
            if std_rgb[c] > 0.0 {
                skewness_rgb[c] = (cube_sum_rgb[c] / count) / std_rgb[c].powi(3);
                kurtosis_rgb[c] = (quad_sum_rgb[c] / count) / std_rgb[c].powi(4) - 3.0;
            }
        }
        
        let saturation_skew = if sat_std > 0.0 {
            (cube_sum_sat / count) / sat_std.powi(3)
        } else {
            0.0
        };
        
        let rg_ratio = if mean_rgb[1] > 0.0 { mean_rgb[0] / mean_rgb[1] } else { 0.0 };
        let bg_ratio = if mean_rgb[1] > 0.0 { mean_rgb[2] / mean_rgb[1] } else { 0.0 };
        
        // Entropy
        let mut entropy = 0.0;
        for &n in hist.iter() {
            if n > 0 {
                let p = n as f32 / count;
                entropy -= p * p.log2();
            }
        }
        
        // Other metrics
        let (quantiles, dynamic_range) = calculate_quantiles_and_dr(img);
        let clipping = calculate_clipping(img);
        let cct_tint = calculate_cct_tint(img);
        let lbp = calculate_lbp(img);
        let glcm = calculate_glcm(img);
        let laplacian_variance = calculate_laplacian_variance(img);
        let psd_slope = calculate_psd_slope(img);
        
        Self {
            mean_rgb,
            std_rgb,
            skewness_rgb,
            kurtosis_rgb,
            quantiles_rgb: quantiles,
            clipping_ratio: clipping,
            entropy,
            dynamic_range,
            lab_mean,
            lab_std,
            rg_ratio,
            bg_ratio,
            saturation_mean: sat_mean,
            saturation_skew,
            cct_tint,
            delta_e: None,
            laplacian_variance,
            psd_slope,
            lbp_hist: lbp,
            glcm_stats: glcm,
            ssim: None,
        }
    }
    
    fn empty() -> Self {
         Self {
            mean_rgb: [0.0; 3],
            std_rgb: [0.0; 3],
            skewness_rgb: [0.0; 3],
            kurtosis_rgb: [0.0; 3],
            quantiles_rgb: [[0; 4]; 3],
            clipping_ratio: [0.0; 2],
            entropy: 0.0,
            dynamic_range: 0.0,
            lab_mean: [0.0; 3],
            lab_std: [0.0; 3],
            rg_ratio: 0.0,
            bg_ratio: 0.0,
            saturation_mean: 0.0,
            saturation_skew: 0.0,
            cct_tint: (0.0, 0.0),
            delta_e: None,
            laplacian_variance: 0.0,
            psd_slope: 0.0,
            lbp_hist: [0.0; 10],
            glcm_stats: [0.0; 4],
            ssim: None,
         }
    }
}

fn get_lum(img: &RgbImage, x: u32, y: u32) -> f32 {
    if x >= img.width() || y >= img.height() {
        return 0.0;
    }
    let p = img.get_pixel(x, y);
    // Rec. 709 luminance
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn calculate_quantiles_and_dr(img: &RgbImage) -> ([[u8; 4]; 3], f32) {
    let mut channels = [Vec::new(), Vec::new(), Vec::new()];
    
    for p in img.pixels() {
        for c in 0..3 {
            channels[c].push(p[c]);
        }
    }
    
    let mut result = [[0; 4]; 3];
    
    // We can estimate dynamic range from Green channel or Luminance
    // Let's use Green channel p99 / p01 for DR
    
    for c in 0..3 {
        channels[c].sort_unstable();
        let len = channels[c].len() as f32;
        if len > 0.0 {
            result[c][0] = channels[c][(len * 0.10) as usize];
            result[c][1] = channels[c][(len * 0.50) as usize];
            result[c][2] = channels[c][(len * 0.90) as usize];
            result[c][3] = channels[c][(len * 0.99) as usize];
        }
    }
    
    // Use Green channel for DR estimation as proxy for luminance
    let g_len = channels[1].len() as f32;
    let dr = if g_len > 0.0 {
        let p01 = channels[1][(g_len * 0.01) as usize] as f32;
        let p99 = channels[1][(g_len * 0.99) as usize] as f32;
        if p01 > 0.0 {
            20.0 * (p99 / p01).log10()
        } else {
            0.0 // Infinite/Undefined
        }
    } else {
        0.0
    };

    (result, dr)
}

fn calculate_clipping(img: &RgbImage) -> [f32; 2] {
    let mut zeros = 0;
    let mut saturated = 0;
    let total = (img.width() * img.height() * 3) as f32;
    
    for p in img.pixels() {
        for c in 0..3 {
            if p[c] == 0 { zeros += 1; }
            if p[c] == 255 { saturated += 1; }
        }
    }
    
    if total > 0.0 {
        [zeros as f32 / total, saturated as f32 / total]
    } else {
        [0.0, 0.0]
    }
}

fn calculate_cct_tint(img: &RgbImage) -> (f32, f32) {
    // McCamy's formula approximation
    // Need XYZ. Assume sRGB input.
    // 1. Calculate Mean RGB (Linear)
    let mut sum_r = 0.0;
    let mut sum_g = 0.0;
    let mut sum_b = 0.0;
    let count = (img.width() * img.height()) as f32;
    
    for p in img.pixels() {
        // Inverse Gamma approx 2.2
        sum_r += (p[0] as f32 / 255.0).powf(2.2);
        sum_g += (p[1] as f32 / 255.0).powf(2.2);
        sum_b += (p[2] as f32 / 255.0).powf(2.2);
    }
    
    let r = sum_r / count;
    let g = sum_g / count;
    let b = sum_b / count;
    
    // RGB to CIE xy
    // sRGB to XYZ matrix (D65)
    let x = 0.4124 * r + 0.3576 * g + 0.1805 * b;
    let y = 0.2126 * r + 0.7152 * g + 0.0722 * b;
    let z = 0.0193 * r + 0.1192 * g + 0.9505 * b;
    
    let sum = x + y + z;
    if sum == 0.0 { return (0.0, 0.0); }
    
    let xe = x / sum;
    let ye = y / sum;
    
    // McCamy's Formula
    let n: f32 = (xe - 0.3320) / (0.1858 - ye);
    let cct = 449.0 * n.powi(3) + 3525.0 * n.powi(2) + 6823.3 * n + 5520.33;
    
    (cct, ye) 
}

#[allow(dead_code)]
fn calculate_delta_e(img1: &RgbImage, img2: &RgbImage) -> f32 {
    let mut sum_de = 0.0;
    let mut count = 0.0;
    
    // Subsample for performance
    let step = 4;
    
    for y in (0..img1.height()).step_by(step) {
        for x in (0..img1.width()).step_by(step) {
            if x < img2.width() && y < img2.height() {
                let p1 = img1.get_pixel(x, y);
                let p2 = img2.get_pixel(x, y);
                
                let c1 = Srgb::new(p1[0] as f32/255.0, p1[1] as f32/255.0, p1[2] as f32/255.0);
                let c2 = Srgb::new(p2[0] as f32/255.0, p2[1] as f32/255.0, p2[2] as f32/255.0);
                
                let l1: Lab = Lab::from_color(c1);
                let l2: Lab = Lab::from_color(c2);
                
                // CIE76 simple Euclidian distance in Lab
                let de = ((l1.l - l2.l).powi(2) + (l1.a - l2.a).powi(2) + (l1.b - l2.b).powi(2)).sqrt();
                sum_de += de;
                count += 1.0;
            }
        }
    }
    
    if count > 0.0 { sum_de / count } else { 0.0 }
}

fn calculate_lbp(img: &RgbImage) -> [f32; 10] {
    let mut hist = [0.0; 10];
    let w = img.width();
    let h = img.height();
    let mut count = 0.0;
    
    for y in 1..h-1 {
        for x in 1..w-1 {
            let c = get_lum(img, x, y);
            let mut code = 0u8;
            
            if get_lum(img, x-1, y-1) >= c { code |= 1; }
            if get_lum(img, x,   y-1) >= c { code |= 2; }
            if get_lum(img, x+1, y-1) >= c { code |= 4; }
            if get_lum(img, x+1, y)   >= c { code |= 8; }
            if get_lum(img, x+1, y+1) >= c { code |= 16; }
            if get_lum(img, x,   y+1) >= c { code |= 32; }
            if get_lum(img, x-1, y+1) >= c { code |= 64; }
            if get_lum(img, x-1, y)   >= c { code |= 128; }
            
            let bin = (code as usize * 10) / 256;
            if bin < 10 {
                hist[bin] += 1.0;
            }
            count += 1.0;
        }
    }
    
    if count > 0.0 {
        for val in &mut hist { *val /= count; }
    }
    
    hist
}

fn calculate_glcm(img: &RgbImage) -> [f32; 4] {
    // Gray Level Co-occurrence Matrix
    // Distance 1, Angle 0 (Horizontal right)
    // Quantize to 16 levels to keep matrix small (16x16)
    
    let mut matrix = [[0.0; 16]; 16];
    let w = img.width();
    let h = img.height();
    let mut count = 0.0;
    
    for y in 0..h {
        for x in 0..w-1 {
            let l1 = (get_lum(img, x, y) / 255.0 * 15.99) as usize;
            let l2 = (get_lum(img, x+1, y) / 255.0 * 15.99) as usize;
            matrix[l1][l2] += 1.0;
            count += 1.0;
        }
    }
    
    // Normalize
    if count > 0.0 {
        for row in &mut matrix {
            for val in row {
                *val /= count;
            }
        }
    }
    
    // Features
    let mut contrast = 0.0;
    let mut correlation = 0.0;
    let mut energy = 0.0;
    let mut homogeneity = 0.0;
    
    let mut mean_i = 0.0;
    let mut mean_j = 0.0;
    
    for (i, row) in matrix.iter().enumerate() {
        for (j, &p) in row.iter().enumerate() {
            mean_i += i as f32 * p;
            mean_j += j as f32 * p;
            
            contrast += (i as f32 - j as f32).powi(2) * p;
            energy += p * p;
            homogeneity += p / (1.0 + (i as f32 - j as f32).abs());
        }
    }
    
    let mut std_i = 0.0;
    let mut std_j = 0.0;
    for (i, row) in matrix.iter().enumerate() {
        for (j, &p) in row.iter().enumerate() {
            std_i += (i as f32 - mean_i).powi(2) * p;
            std_j += (j as f32 - mean_j).powi(2) * p;
        }
    }
    std_i = std_i.sqrt();
    std_j = std_j.sqrt();
    
    if std_i * std_j > 0.0 {
        for (i, row) in matrix.iter().enumerate() {
            for (j, &p) in row.iter().enumerate() {
                correlation += ((i as f32 - mean_i) * (j as f32 - mean_j) * p) / (std_i * std_j);
            }
        }
    }
    
    [contrast, correlation, energy, homogeneity]
}

fn calculate_laplacian_variance(img: &RgbImage) -> f32 {
    let w = img.width();
    let h = img.height();
    let mut sum = 0.0;
    let mut sq_sum = 0.0;
    let mut count = 0.0;
    
    // Convolve with 3x3 Laplacian kernel
    // [ 0  1  0]
    // [ 1 -4  1]
    // [ 0  1  0]
    
    for y in 1..h-1 {
        for x in 1..w-1 {
            let c = get_lum(img, x, y);
            let u = get_lum(img, x, y-1);
            let d = get_lum(img, x, y+1);
            let l = get_lum(img, x-1, y);
            let r = get_lum(img, x+1, y);
            
            let val = u + d + l + r - 4.0 * c;
            sum += val;
            sq_sum += val * val;
            count += 1.0;
        }
    }
    
    if count > 0.0 {
        let mean = sum / count;
        (sq_sum / count) - (mean * mean)
    } else {
        0.0
    }
}

fn calculate_psd_slope(img: &RgbImage) -> f32 {
    // 1D Radial PSD Slope calculation
    // 1. Resize to 256x256 (Power of 2)
    // 2. FFT
    // 3. Radial Average
    // 4. Linear Regression on log-log
    
    let size = 256;
    let resized = image::imageops::resize(img, size, size, image::imageops::FilterType::Triangle);
    
    let mut input: Vec<Complex<f32>> = Vec::with_capacity((size * size) as usize);
    for y in 0..size {
        for x in 0..size {
            let val = get_lum(&resized, x, y);
            input.push(Complex::new(val, 0.0));
        }
    }
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(size as usize * size as usize);
    
    fft.process(&mut input);
    
    // This is 1D FFT of flattened array, which is NOT 2D FFT.
    // RustFFT is 1D. For 2D we need to do rows then columns.
    
    // Correct 2D FFT implementation with RustFFT:
    // 1. FFT rows
    // 2. Transpose
    // 3. FFT rows (originally columns)
    // 4. Transpose back
    
    let mut buffer = vec![Complex::new(0.0, 0.0); (size * size) as usize];
    let fft_row = planner.plan_fft_forward(size as usize);
    
    // FFT Rows
    for y in 0..size {
        let start = (y * size) as usize;
        let end = start + size as usize;
        fft_row.process(&mut input[start..end]);
    }
    
    // Transpose
    for y in 0..size {
        for x in 0..size {
            buffer[(x * size + y) as usize] = input[(y * size + x) as usize];
        }
    }
    
    // FFT Cols (Rows of transposed)
    for x in 0..size {
        let start = (x * size) as usize;
        let end = start + size as usize;
        fft_row.process(&mut buffer[start..end]);
    }
    
    // Calculate Power Spectrum & Radial Average
    let mut radial_sum = vec![0.0; (size / 2) as usize];
    let mut radial_count = vec![0.0; (size / 2) as usize];
    
    for y in 0..size {
        for x in 0..size {
            // Shifted coordinates
            let dy = y as f32 - if y < size/2 { 0.0 } else { size as f32 };
            let dx = x as f32 - if x < size/2 { 0.0 } else { size as f32 };
            
            let dist = (dx*dx + dy*dy).sqrt();
            let idx = dist as usize;
            
            if idx > 0 && idx < (size/2) as usize {
                let amp = buffer[(x * size + y) as usize].norm_sqr();
                radial_sum[idx] += amp;
                radial_count[idx] += 1.0;
            }
        }
    }
    
    // Linear Regression: log(P) = beta * log(f) + C
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;
    let mut n = 0.0;
    
    for i in 1..(size/2) as usize {
        if radial_count[i] > 0.0 {
            let freq = i as f32;
            let power = radial_sum[i] / radial_count[i];
            
            if power > 0.0 {
                let log_f = freq.ln();
                let log_p = power.ln();
                
                sum_x += log_f;
                sum_y += log_p;
                sum_xy += log_f * log_p;
                sum_xx += log_f * log_f;
                n += 1.0;
            }
        }
    }
    
    if n > 1.0 {
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
        -slope // Return positive beta (1/f^beta)
    } else {
        0.0
    }
}
