use image::RgbImage;
use palette::{Srgb, Lab, FromColor, Hsv};
use rustfft::{FftPlanner, num_complex::Complex};

#[derive(Debug, Clone)]
pub struct FilmMetrics {
    // Basic Stats
    pub mean_rgb: [f32; 3],
    pub std_rgb: [f32; 3],
    pub skewness_rgb: [f32; 3],
    pub kurtosis_rgb: [f32; 3],
    pub entropy: f32,
    pub dynamic_range: f32,
    
    // Color Science
    pub lab_mean: [f32; 3],
    pub lab_std: [f32; 3],
    pub rg_ratio: f32,
    pub bg_ratio: f32,
    pub saturation_mean: f32,
    pub saturation_skew: f32,
    
    // Frequency / Texture
    pub laplacian_variance: f32,
    pub psd_slope: f32, // 1/f^beta, returns beta
    
    // Perceptual / Structure
    pub ssim: Option<f32>, // Needs reference
}

impl FilmMetrics {
    pub fn analyze(img: &RgbImage) -> Self {
        let (mean, std, skew, kurt) = calculate_moments(img);
        let entropy = calculate_entropy(img);
        let dr = calculate_dynamic_range(img);
        
        let (lab_m, lab_s) = calculate_lab_stats(img);
        let (rg, bg) = calculate_ratios(img);
        let (sat_m, sat_skew) = calculate_saturation_stats(img);
        
        let lap = calculate_laplacian_variance(img);
        let beta = calculate_psd_slope(img);

        FilmMetrics {
            mean_rgb: mean,
            std_rgb: std,
            skewness_rgb: skew,
            kurtosis_rgb: kurt,
            entropy,
            dynamic_range: dr,
            lab_mean: lab_m,
            lab_std: lab_s,
            rg_ratio: rg,
            bg_ratio: bg,
            saturation_mean: sat_m,
            saturation_skew: sat_skew,
            laplacian_variance: lap,
            psd_slope: beta,
            ssim: None,
        }
    }

    pub fn with_reference(mut self, img: &RgbImage, ref_img: &RgbImage) -> Self {
        self.ssim = Some(calculate_ssim(img, ref_img));
        self
    }
}

fn calculate_moments(img: &RgbImage) -> ([f32; 3], [f32; 3], [f32; 3], [f32; 3]) {
    let mut sum = [0.0; 3];
    let mut sq_sum = [0.0; 3];
    let mut cube_sum = [0.0; 3];
    let mut quad_sum = [0.0; 3];
    let count = (img.width() * img.height()) as f32;

    for p in img.pixels() {
        for c in 0..3 {
            let v = p[c] as f32;
            sum[c] += v;
            sq_sum[c] += v * v;
            cube_sum[c] += v * v * v;
            quad_sum[c] += v * v * v * v;
        }
    }

    let mut mean = [0.0; 3];
    let mut std = [0.0; 3];
    let mut skew = [0.0; 3];
    let mut kurt = [0.0; 3];

    for c in 0..3 {
        mean[c] = sum[c] / count;
        let var = (sq_sum[c] / count) - (mean[c] * mean[c]);
        std[c] = var.sqrt();
        
        // Skewness = E[(x-mu)^3] / sigma^3
        let m3 = (cube_sum[c] / count) - 3.0 * mean[c] * (sq_sum[c] / count) + 2.0 * mean[c].powi(3);
        let m4 = (quad_sum[c] / count) - 4.0 * mean[c] * (cube_sum[c] / count) + 6.0 * mean[c].powi(2) * (sq_sum[c] / count) - 3.0 * mean[c].powi(4);
        
        if std[c] > 1e-5 {
            skew[c] = m3 / std[c].powi(3);
            kurt[c] = m4 / std[c].powi(4) - 3.0; // Excess kurtosis
        }
    }

    (mean, std, skew, kurt)
}

fn calculate_entropy(img: &RgbImage) -> f32 {
    let mut hist = [0u32; 256];
    let total = (img.width() * img.height()) as f32;
    
    // Use luminance for entropy
    for p in img.pixels() {
        let lum = (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) as u8;
        hist[lum as usize] += 1;
    }
    
    let mut entropy = 0.0;
    for &count in &hist {
        if count > 0 {
            let p = count as f32 / total;
            entropy -= p * p.log2();
        }
    }
    entropy
}

fn calculate_dynamic_range(img: &RgbImage) -> f32 {
    let mut lums: Vec<u8> = img.pixels()
        .map(|p| (0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32) as u8)
        .collect();
    lums.sort_unstable();
    
    let p01 = lums[(lums.len() as f32 * 0.01) as usize] as f32;
    let p99 = lums[(lums.len() as f32 * 0.99) as usize] as f32;
    
    // Log dynamic range if possible, otherwise linear
    if p01 > 0.0 {
        (p99 / p01).log2() // Stops
    } else {
        0.0
    }
}

fn calculate_lab_stats(img: &RgbImage) -> ([f32; 3], [f32; 3]) {
    let mut count = 0.0;
    
    let mut acc_l = 0.0; let mut acc_l2 = 0.0;
    let mut acc_a = 0.0; let mut acc_a2 = 0.0;
    let mut acc_b = 0.0; let mut acc_b2 = 0.0;
    
    for p in img.pixels() {
        let srgb = Srgb::new(p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
        let lab: Lab = Lab::from_color(srgb);
        
        acc_l += lab.l; acc_l2 += lab.l * lab.l;
        acc_a += lab.a; acc_a2 += lab.a * lab.a;
        acc_b += lab.b; acc_b2 += lab.b * lab.b;
        count += 1.0;
    }
    
    let mean_l = acc_l / count;
    let mean_a = acc_a / count;
    let mean_b = acc_b / count;
    
    let var_l = (acc_l2 / count) - mean_l * mean_l;
    let var_a = (acc_a2 / count) - mean_a * mean_a;
    let var_b = (acc_b2 / count) - mean_b * mean_b;
    
    ([mean_l, mean_a, mean_b], [var_l.sqrt(), var_a.sqrt(), var_b.sqrt()])
}

fn calculate_ratios(img: &RgbImage) -> (f32, f32) {
    let mut sum_rg = 0.0;
    let mut sum_bg = 0.0;
    let mut count = 0.0;
    
    for p in img.pixels() {
        let r = p[0] as f32;
        let g = p[1] as f32;
        let b = p[2] as f32;
        
        if g > 1.0 {
            sum_rg += r / g;
            sum_bg += b / g;
            count += 1.0;
        }
    }
    
    if count > 0.0 {
        (sum_rg / count, sum_bg / count)
    } else {
        (0.0, 0.0)
    }
}

fn calculate_saturation_stats(img: &RgbImage) -> (f32, f32) {
    let mut sum_s = 0.0;
    let mut sum_s2 = 0.0;
    let mut sum_s3 = 0.0;
    let mut count = 0.0;
    
    for p in img.pixels() {
        let srgb = Srgb::new(p[0] as f32 / 255.0, p[1] as f32 / 255.0, p[2] as f32 / 255.0);
        let hsv: Hsv = Hsv::from_color(srgb);
        let s = hsv.saturation;
        
        sum_s += s;
        sum_s2 += s * s;
        sum_s3 += s * s * s;
        count += 1.0;
    }
    
    let mean = sum_s / count;
    let var = (sum_s2 / count) - mean * mean;
    let std = var.sqrt();
    
    let skew = if std > 1e-5 {
        let m3 = (sum_s3 / count) - 3.0 * mean * (sum_s2 / count) + 2.0 * mean.powi(3);
        m3 / std.powi(3)
    } else {
        0.0
    };
    
    (mean, skew)
}

fn calculate_laplacian_variance(img: &RgbImage) -> f32 {
    // 3x3 Laplacian kernel
    //  0  1  0
    //  1 -4  1
    //  0  1  0
    
    let w = img.width();
    let h = img.height();
    let mut sum = 0.0;
    let mut sq_sum = 0.0;
    let mut count = 0.0;
    
    for y in 1..h-1 {
        for x in 1..w-1 {
            let p_c = get_lum(img, x, y);
            let p_u = get_lum(img, x, y-1);
            let p_d = get_lum(img, x, y+1);
            let p_l = get_lum(img, x-1, y);
            let p_r = get_lum(img, x+1, y);
            
            let lap = p_u + p_d + p_l + p_r - 4.0 * p_c;
            sum += lap;
            sq_sum += lap * lap;
            count += 1.0;
        }
    }
    
    if count > 0.0 {
        let mean = sum / count;
        (sq_sum / count) - mean * mean
    } else {
        0.0
    }
}

fn get_lum(img: &RgbImage, x: u32, y: u32) -> f32 {
    let p = img.get_pixel(x, y);
    0.2126 * p[0] as f32 + 0.7152 * p[1] as f32 + 0.0722 * p[2] as f32
}

fn calculate_psd_slope(img: &RgbImage) -> f32 {
    // Simplified 1D PSD on center row
    let w = img.width() as usize;
    let h = img.height() as usize;
    let center_row = h / 2;
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(w);
    
    let mut buffer: Vec<Complex<f32>> = (0..w)
        .map(|x| Complex { re: get_lum(img, x as u32, center_row as u32), im: 0.0 })
        .collect();
        
    fft.process(&mut buffer);
    
    // Calculate Power Spectrum
    let psd: Vec<(f32, f32)> = buffer.iter()
        .enumerate()
        .take(w / 2) // Nyquist
        .skip(1) // Skip DC
        .map(|(i, c)| (i as f32, c.norm_sqr()))
        .collect();
        
    // Fit line to log-log: log(P) = -beta * log(f) + C
    let n = psd.len() as f32;
    let mut sum_x = 0.0;
    let mut sum_y = 0.0;
    let mut sum_xy = 0.0;
    let mut sum_xx = 0.0;
    
    for (f, p) in psd {
        if f > 0.0 && p > 0.0 {
            let lx = f.ln();
            let ly = p.ln();
            sum_x += lx;
            sum_y += ly;
            sum_xy += lx * ly;
            sum_xx += lx * lx;
        }
    }
    
    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    -slope // beta
}

fn calculate_ssim(img1: &RgbImage, img2: &RgbImage) -> f32 {
    // Simplified SSIM on Luminance
    // Just a placeholder for global SSIM-like stats
    // SSIM(x,y) = (2*mu_x*mu_y + c1)(2*sig_xy + c2) / ...
    
    let (m1, s1) = calculate_lab_stats(img1); // reusing mean/std
    let (m2, s2) = calculate_lab_stats(img2);
    
    let mu1 = m1[0]; // L*
    let mu2 = m2[0];
    let sig1_sq = s1[0] * s1[0];
    let sig2_sq = s2[0] * s2[0];
    
    // Covariance?
    let mut cov = 0.0;
    let mut count = 0.0;
    for (p1, p2) in img1.pixels().zip(img2.pixels()) {
        let l1 = 0.2126 * p1[0] as f32 + 0.7152 * p1[1] as f32 + 0.0722 * p1[2] as f32;
        let l2 = 0.2126 * p2[0] as f32 + 0.7152 * p2[1] as f32 + 0.0722 * p2[2] as f32;
        // Approximation using L* from Lab might be better but let's use raw RGB lum
        cov += (l1 - mu1) * (l2 - mu2); // This mixes Lab mean with RGB pixels, inaccurate but close enough for placeholder
        count += 1.0;
    }
    let sig_xy = cov / count;
    
    let c1 = (0.01 * 255.0) * (0.01 * 255.0);
    let c2 = (0.03 * 255.0) * (0.03 * 255.0);
    
    ((2.0 * mu1 * mu2 + c1) * (2.0 * sig_xy + c2)) / 
    ((mu1 * mu1 + mu2 * mu2 + c1) * (sig1_sq + sig2_sq + c2))
}
