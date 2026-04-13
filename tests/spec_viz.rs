use filmr::spectral::{CameraSensitivities, Spectrum, BINS, LAMBDA_START, LAMBDA_STEP};

fn print_spectrum(label: &str, r: f32, g: f32, b: f32) {
    let cam = CameraSensitivities::srgb();
    let d65 = Spectrum::new_d65();
    let spec = cam.uplift(r, g, b);

    // Also show with D65 illuminant
    eprintln!("\n=== {} (R={}, G={}, B={}) ===", label, r, g, b);
    eprintln!("nm   | uplift | ×D65   | ▌bar");
    eprintln!("-----|--------|--------|----");

    let mut max_val = 0.0f32;
    let mut vals = vec![];
    for i in 0..BINS {
        let v = spec.power[i] * d65.power[i];
        max_val = max_val.max(v);
        vals.push(v);
    }

    for (i, (&raw, &scaled)) in spec.power.iter().zip(vals.iter()).enumerate() {
        let nm = LAMBDA_START + i * LAMBDA_STEP;
        let bar_len = if max_val > 0.0 {
            (scaled / max_val * 40.0) as usize
        } else {
            0
        };
        let bar: String = "█".repeat(bar_len);

        // Only print every 2nd bin to keep it readable
        if i % 2 == 0 {
            eprintln!("{:3}nm | {:6.3} | {:6.1} | {}", nm, raw, scaled, bar);
        }
    }
}

#[test]
fn visualize_spectra() {
    print_spectrum("White", 1.0, 1.0, 1.0);
    print_spectrum("Pure Red", 1.0, 0.0, 0.0);
    print_spectrum("Pure Green", 0.0, 1.0, 0.0);
    print_spectrum("Pure Blue", 0.0, 0.0, 1.0);
    print_spectrum("Warm Orange", 0.9, 0.5, 0.2);
    print_spectrum("Sky Blue", 0.3, 0.6, 0.9);
}
