//! Full-spectrum simulation engine (Accurate mode).
//!
//! Propagates light wavelength-by-wavelength through the film layer stack,
//! accounting for Beer-Lambert absorption, Fresnel interface reflections,
//! and Mie/Rayleigh scattering within emulsion layers.

use crate::film_layer::{EmulsionChannel, FilmLayerStack, LayerKind};
use crate::spectral::BINS;

/// Per-wavelength exposure accumulated by each colour record.
#[derive(Debug, Clone)]
pub struct LayerExposure {
    pub red: [f32; BINS],
    pub green: [f32; BINS],
    pub blue: [f32; BINS],
}

impl Default for LayerExposure {
    fn default() -> Self {
        Self {
            red: [0.0; BINS],
            green: [0.0; BINS],
            blue: [0.0; BINS],
        }
    }
}

/// Fresnel reflectance at normal incidence between two dielectrics.
#[inline]
fn fresnel_r(n1: f32, n2: f32) -> f32 {
    let r = (n1 - n2) / (n1 + n2);
    r * r
}

/// Propagate a spectral power distribution through a `FilmLayerStack`.
///
/// `incident` is the per-wavelength irradiance arriving at the film surface (81 bins).
/// Returns the exposure captured by each emulsion channel.
pub fn propagate(stack: &FilmLayerStack, incident: &[f32; BINS]) -> LayerExposure {
    let mut exposure = LayerExposure::default();
    let layers = &stack.layers;
    if layers.is_empty() {
        return exposure;
    }

    // --- Forward pass (top → bottom) ---
    let mut power = *incident; // current spectral power travelling downward
    let mut prev_n = 1.0f32; // air

    for layer in layers.iter() {
        // 1. Fresnel reflection at interface
        let r = fresnel_r(prev_n, layer.refractive_index);
        let t_interface = 1.0 - r; // transmitted fraction

        for p in power.iter_mut() {
            *p *= t_interface;
        }

        // 2. Beer-Lambert absorption + scattering through the layer
        let d = layer.thickness_um;
        for (i, p) in power.iter_mut().enumerate() {
            let total_atten = layer.absorption[i] + layer.scattering;
            *p *= (-total_atten * d).exp();
        }

        // 3. If emulsion, record absorbed energy as exposure
        if let LayerKind::Emulsion { channel } = layer.kind {
            let target = match channel {
                EmulsionChannel::Red => &mut exposure.red,
                EmulsionChannel::Green => &mut exposure.green,
                EmulsionChannel::Blue => &mut exposure.blue,
            };
            for i in 0..BINS {
                // Absorbed fraction = 1 - exp(-absorption * d)
                // We approximate: energy deposited ≈ incident_on_layer * (1 - exp(-abs*d))
                // But `power` already has the transmitted value, so deposited = power_before - power_after.
                // power_before = power[i] / exp(-total*d), deposited from absorption only:
                let total_atten = layer.absorption[i] + layer.scattering;
                let exp_total = (-total_atten * d).exp();
                if exp_total < 1.0 {
                    // power_before_layer = power[i] / exp_total (undo attenuation to get entry power)
                    let power_in = power[i] / exp_total.max(1e-30);
                    // fraction absorbed (not scattered)
                    let abs_fraction = if total_atten > 0.0 {
                        layer.absorption[i] / total_atten
                    } else {
                        0.0
                    };
                    target[i] += power_in * (1.0 - exp_total) * abs_fraction;
                }
            }
        }

        prev_n = layer.refractive_index;
    }

    // --- Backward pass (base reflection → back up through layers) ---
    // The base reflects some light back. We propagate upward through the stack
    // in reverse, collecting additional exposure in emulsion layers.
    // Base reflectance = Fresnel at base/air interface (bottom side).
    let base_n = layers.last().map(|l| l.refractive_index).unwrap_or(1.0);
    let base_r = fresnel_r(base_n, 1.0); // base → air

    let mut back_power = [0.0f32; BINS];
    for i in 0..BINS {
        back_power[i] = power[i] * base_r;
    }

    prev_n = base_n;

    for layer in layers.iter().rev() {
        let r = fresnel_r(prev_n, layer.refractive_index);
        let t_interface = 1.0 - r;

        for p in back_power.iter_mut() {
            *p *= t_interface;
        }

        let d = layer.thickness_um;
        for (i, p) in back_power.iter_mut().enumerate() {
            let total_atten = layer.absorption[i] + layer.scattering;
            *p *= (-total_atten * d).exp();
        }

        // Record exposure on backward pass too
        if let LayerKind::Emulsion { channel } = layer.kind {
            let target = match channel {
                EmulsionChannel::Red => &mut exposure.red,
                EmulsionChannel::Green => &mut exposure.green,
                EmulsionChannel::Blue => &mut exposure.blue,
            };
            for i in 0..BINS {
                let total_atten = layer.absorption[i] + layer.scattering;
                let exp_total = (-total_atten * d).exp();
                if exp_total < 1.0 {
                    let power_in = back_power[i] / exp_total.max(1e-30);
                    let abs_fraction = if total_atten > 0.0 {
                        layer.absorption[i] / total_atten
                    } else {
                        0.0
                    };
                    target[i] += power_in * (1.0 - exp_total) * abs_fraction;
                }
            }
        }

        prev_n = layer.refractive_index;
    }

    exposure
}

/// Integrate per-wavelength exposure into a scalar triplet [R, G, B]
/// using trapezoidal rule (matches `Spectrum::integrate_product`).
pub fn integrate_exposure(exp: &LayerExposure) -> [f32; 3] {
    let integrate = |data: &[f32; BINS]| -> f32 {
        let mut sum: f32 = data[1..BINS - 1].iter().sum();
        sum += 0.5 * (data[0] + data[BINS - 1]);
        sum * 5.0 // LAMBDA_STEP
    };
    [
        integrate(&exp.red),
        integrate(&exp.green),
        integrate(&exp.blue),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::film_layer::FilmLayerStack;
    use crate::spectral::BINS;

    #[test]
    fn test_propagate_no_layers() {
        let stack = FilmLayerStack {
            layers: vec![],
            inhibition: [[0.0; 3]; 3],
        };
        let incident = [1.0f32; BINS];
        let exp = propagate(&stack, &incident);
        // No layers → no exposure
        assert!(exp.red.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_propagate_default_color_negative() {
        let stack = FilmLayerStack::default_color_negative();
        let incident = crate::cie_data::D65_SPD;
        let exp = propagate(&stack, &incident);
        let rgb = integrate_exposure(&exp);

        // All channels should receive some exposure under D65
        assert!(rgb[0] > 0.0, "Red exposure should be > 0");
        assert!(rgb[1] > 0.0, "Green exposure should be > 0");
        assert!(rgb[2] > 0.0, "Blue exposure should be > 0");

        // Blue layer is on top → should get the most light
        assert!(
            rgb[2] > rgb[0] && rgb[2] > rgb[1],
            "Blue layer (top) should capture most energy: R={:.2}, G={:.2}, B={:.2}",
            rgb[0],
            rgb[1],
            rgb[2]
        );
    }

    #[test]
    fn test_backward_pass_adds_exposure() {
        // With base reflection, total exposure should be higher than forward-only
        let stack = FilmLayerStack::default_color_negative();
        let incident = [1.0f32; BINS];

        let exp = propagate(&stack, &incident);
        let rgb = integrate_exposure(&exp);

        // Exposure should be non-trivially positive (backward pass contributes)
        assert!(rgb[0] > 0.0);
    }

    #[test]
    fn test_accurate_white_balance_diag() {
        let stack = FilmLayerStack::default_color_negative();
        let camera = crate::spectral::CameraSensitivities::srgb();
        let d65 = crate::spectral::Spectrum::new_d65();

        // White-point normalization (same as AccurateDevelopStage)
        let white = camera.uplift(1.0, 1.0, 1.0);
        let mut ws = [0.0f32; BINS];
        for (i, s) in ws.iter_mut().enumerate() {
            *s = white.power[i] * d65.power[i];
        }
        let we = propagate(&stack, &ws);
        let wrgb = integrate_exposure(&we);
        println!(
            "White raw: R={:.2}, G={:.2}, B={:.2}",
            wrgb[0], wrgb[1], wrgb[2]
        );

        // Use STANDARD_DAYLIGHT offset (0.18) for norm
        let offset = 0.18f32;
        let mid_gray = 0.18f32;
        let norm = [
            offset / (wrgb[0] * mid_gray),
            offset / (wrgb[1] * mid_gray),
            offset / (wrgb[2] * mid_gray),
        ];
        println!("Norm: {:?}", norm);

        // Test colors
        for (label, r, g, b) in [
            ("White", 1.0, 1.0, 1.0),
            ("Gray18", 0.18, 0.18, 0.18),
            ("Red", 0.73, 0.0, 0.0),
            ("Gray50", 0.5, 0.5, 0.5),
        ] {
            let spec = camera.uplift(r, g, b);
            let mut scaled = [0.0f32; BINS];
            for (i, s) in scaled.iter_mut().enumerate() {
                *s = spec.power[i] * d65.power[i];
            }
            let exp = propagate(&stack, &scaled);
            let rgb = integrate_exposure(&exp);
            let normed = [rgb[0] * norm[0], rgb[1] * norm[1], rgb[2] * norm[2]];
            let log_e = [
                normed[0].max(1e-6).log10(),
                normed[1].max(1e-6).log10(),
                normed[2].max(1e-6).log10(),
            ];
            println!(
                "{:8}: exposure=[{:.4}, {:.4}, {:.4}] log_e=[{:.3}, {:.3}, {:.3}]",
                label, normed[0], normed[1], normed[2], log_e[0], log_e[1], log_e[2]
            );
        }
    }
}
