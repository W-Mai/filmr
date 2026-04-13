//! Film multi-layer structure model.
//!
//! Describes the physical stack of layers that light traverses in a film strip.
//! Used by the full-spectrum engine (Accurate mode) for per-wavelength propagation.

use crate::spectral::BINS;
use serde::{Deserialize, Serialize};
use serde_big_array::BigArray;

/// A single layer in the film stack.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmLayer {
    pub name: String,
    pub kind: LayerKind,
    /// Thickness in micrometers.
    pub thickness_um: f32,
    /// Refractive index (real part, ~1.5 for gelatin).
    pub refractive_index: f32,
    /// Spectral absorption coefficient per µm, 81 bins (380-780nm, 5nm).
    #[serde(with = "BigArray")]
    pub absorption: [f32; BINS],
    /// Scattering coefficient per µm (Mie/Rayleigh in emulsion).
    pub scattering: f32,
    /// Dye spectral absorption curve (normalized, 81 bins).
    /// For emulsion layers: the dye formed during development (Yellow/Magenta/Cyan).
    /// Density D at wavelength λ: D(λ) = density_scalar × dye_spectrum[λ].
    /// None = use simple per-channel density (legacy behavior).
    #[serde(skip)]
    pub dye_spectrum: Option<[f32; BINS]>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LayerKind {
    Overcoat,
    Emulsion { channel: EmulsionChannel },
    YellowFilter,
    Interlayer,
    AntiHalation,
    Base,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EmulsionChannel {
    Blue,
    Green,
    Red,
}

/// The complete layer stack, ordered top (light-entry) to bottom.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilmLayerStack {
    pub layers: Vec<FilmLayer>,
    /// Interlayer interimage effect (developer inhibition).
    /// `inhibition[i][j]` = how much density in channel j suppresses channel i.
    /// Channels: 0=Red, 1=Green, 2=Blue.  Diagonal should be 0.
    /// Typical values: 0.05–0.15 for colour negative.
    pub inhibition: [[f32; 3]; 3],
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

pub fn gaussian_absorption(center_nm: f32, sigma_nm: f32, amplitude: f32) -> [f32; BINS] {
    let mut a = [0.0f32; BINS];
    for (i, v) in a.iter_mut().enumerate() {
        let lambda = 380.0 + i as f32 * 5.0;
        let d = (lambda - center_nm) / sigma_nm;
        *v = amplitude * (-0.5 * d * d).exp();
    }
    a
}

fn add_absorption(a: &[f32; BINS], b: &[f32; BINS]) -> [f32; BINS] {
    let mut out = [0.0f32; BINS];
    for i in 0..BINS {
        out[i] = a[i] + b[i];
    }
    out
}

// ---------------------------------------------------------------------------
// Dye spectral absorption curves (normalized to peak = 1.0)
// ---------------------------------------------------------------------------

/// Yellow dye: absorbs blue light. Formed in the blue-sensitive layer.
pub fn dye_yellow() -> [f32; BINS] {
    // Main absorption peak ~440nm
    gaussian_absorption(440.0, 30.0, 1.0)
}

/// Magenta dye: absorbs green light. Formed in the green-sensitive layer.
/// Has a secondary absorption bump in the blue (~430nm).
pub fn dye_magenta() -> [f32; BINS] {
    add_absorption(
        &gaussian_absorption(550.0, 40.0, 1.0),
        &gaussian_absorption(430.0, 20.0, 0.3),
    )
}

/// Cyan dye: absorbs red light. Formed in the red-sensitive layer.
/// Has a secondary absorption bump in the blue (~430nm).
pub fn dye_cyan() -> [f32; BINS] {
    add_absorption(
        &gaussian_absorption(650.0, 50.0, 1.0),
        &gaussian_absorption(430.0, 25.0, 0.2),
    )
}

const fn flat(v: f32) -> [f32; BINS] {
    [v; BINS]
}

// ---------------------------------------------------------------------------
// Default stacks + builder methods
// ---------------------------------------------------------------------------

impl FilmLayerStack {
    /// Generic colour-negative (Portra-like).
    pub fn default_color_negative() -> Self {
        Self {
            inhibition: [
                //        R      G      B     ← source of inhibition
                [0.00, -0.08, -0.04], // → Red   (suppressed by G and B development)
                [-0.06, 0.00, -0.06], // → Green (suppressed by R and B)
                [-0.04, -0.08, 0.00], // → Blue  (suppressed by R and G)
            ],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: flat(0.0),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Blue Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Blue,
                    },
                    thickness_um: 6.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(450.0, 30.0, 0.12),
                    scattering: 0.02,
                    dye_spectrum: Some(dye_yellow()),
                },
                FilmLayer {
                    name: "Yellow Filter".into(),
                    kind: LayerKind::YellowFilter,
                    thickness_um: 1.0,
                    refractive_index: 1.52,
                    absorption: gaussian_absorption(440.0, 35.0, 0.8),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Green Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 5.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(550.0, 35.0, 0.10),
                    scattering: 0.02,
                    dye_spectrum: Some(dye_magenta()),
                },
                FilmLayer {
                    name: "Interlayer".into(),
                    kind: LayerKind::Interlayer,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: flat(0.0),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Red Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Red,
                    },
                    thickness_um: 5.0,
                    refractive_index: 1.53,
                    absorption: add_absorption(
                        &gaussian_absorption(640.0, 40.0, 0.10),
                        &gaussian_absorption(440.0, 25.0, 0.02),
                    ),
                    scattering: 0.02,
                    dye_spectrum: Some(dye_cyan()),
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(600.0, 120.0, 0.5),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 125.0,
                    refractive_index: 1.65,
                    absorption: flat(0.001),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }
    }

    /// Generic B&W panchromatic negative.
    pub fn default_bw_negative() -> Self {
        Self {
            inhibition: [[0.0; 3]; 3],
            layers: vec![
                FilmLayer {
                    name: "Overcoat".into(),
                    kind: LayerKind::Overcoat,
                    thickness_um: 1.0,
                    refractive_index: 1.50,
                    absorption: flat(0.0),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Panchromatic Emulsion".into(),
                    kind: LayerKind::Emulsion {
                        channel: EmulsionChannel::Green,
                    },
                    thickness_um: 8.0,
                    refractive_index: 1.53,
                    absorption: gaussian_absorption(540.0, 100.0, 0.08),
                    scattering: 0.03,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Anti-Halation".into(),
                    kind: LayerKind::AntiHalation,
                    thickness_um: 2.0,
                    refractive_index: 1.50,
                    absorption: gaussian_absorption(580.0, 120.0, 0.4),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
                FilmLayer {
                    name: "Base".into(),
                    kind: LayerKind::Base,
                    thickness_um: 125.0,
                    refractive_index: 1.65,
                    absorption: flat(0.001),
                    scattering: 0.0,
                    dye_spectrum: None,
                },
            ],
        }
    }

    // --- Builder / chain methods ---

    /// Override inhibition matrix.
    pub fn with_inhibition(mut self, inhibition: [[f32; 3]; 3]) -> Self {
        self.inhibition = inhibition;
        self
    }

    /// Scale all emulsion scattering coefficients.
    pub fn with_scatter_scale(mut self, factor: f32) -> Self {
        for layer in &mut self.layers {
            if matches!(layer.kind, LayerKind::Emulsion { .. }) {
                layer.scattering *= factor;
            }
        }
        self
    }

    /// Scale all emulsion thicknesses.
    pub fn with_thickness_scale(mut self, factor: f32) -> Self {
        for layer in &mut self.layers {
            if matches!(layer.kind, LayerKind::Emulsion { .. }) {
                layer.thickness_um *= factor;
            }
        }
        self
    }

    /// Scale all emulsion absorption amplitudes.
    pub fn with_absorption_scale(mut self, factor: f32) -> Self {
        for layer in &mut self.layers {
            if matches!(layer.kind, LayerKind::Emulsion { .. }) {
                for v in &mut layer.absorption {
                    *v *= factor;
                }
            }
        }
        self
    }

    /// Replace a specific layer by kind (first match).
    pub fn with_layer(mut self, new_layer: FilmLayer) -> Self {
        if let Some(l) = self.layers.iter_mut().find(|l| l.kind == new_layer.kind) {
            *l = new_layer;
        }
        self
    }

    /// Remove anti-halation layer (e.g. for slide film derived from negative base).
    pub fn without_antihalation(mut self) -> Self {
        self.layers.retain(|l| l.kind != LayerKind::AntiHalation);
        self
    }
}
