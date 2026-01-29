# Validation Framework

To pass **industrial-grade film simulation certification**, a measurement standard from **molecular scale to perceptual scale** must be established. Below is the complete **7-layer verification system**, with quantifiable thresholds and Rust-readable test protocols for each layer:

---

## Layer 0: Spectral Fidelity

### 0.1 Emulsion Spectral Sensitivity $S(\lambda)$
**Standard**: ISO 5800:2001 / ISO 6846
**Measurement Object**: Quantum efficiency of Blue/Green/Red sensitive layers to monochromatic light
**Data Format**: 380–780 nm, 5 nm step, 81-dimensional vector
**Pass Thresholds**:
- Peak wavelength error $< \pm 5$ nm
- Full Width at Half Maximum (FWHM) error $< \pm 15$ nm
- Interlayer crosstalk (Blue/Green sensitivity ratio at 480 nm) $< 15\%$

**Reference Data Source**:
```rust
// Kodak Portra 400 Official Sensitivity (Pre-normalization)
const S_BLUE: [f32; 81] = [0.01, 0.03, ..., 0.95, 0.02]; // 380-780nm, 5nm step
const S_GREEN: [f32; 81] = [...];
const S_RED: [f32; 81] = [...];
```

### 0.2 Dye Spectral Absorption Cross-section $\varepsilon(\lambda)$
**Standard**: Status A Densitometry Calibration (ISO 5-3)
**Measurement Object**: Absorption spectra of Yellow/Magenta/Cyan dyes at maximum density
**Pass Thresholds**:
- Peak absorbance correlation with Kodak patent data $R^2 > 0.995$
- Dye overlap (Yellow dye absorbance at 550 nm $< 12\%$ of peak)

---

## Layer 1: Exposure Response

### 1.1 H-D Curve (Hurter-Driffield)
**Standard**: ISO 6 (B&W) / ISO 5800 (Color)
**Measurement Object**: $\log_{10}(\text{Exposure})$ vs Density $D$
**Key Parameters**:
| Parameter | Symbol | Acceptable Range | Measurement Method |
|-----------|--------|------------------|-------------------|
| Fog Density | $D_{\min}$ | $0.15 \pm 0.03$ | Unexposed base |
| Max Density | $D_{\max}$ | $> 2.8$ | Overexposed by 5 stops |
| Contrast | $\gamma$ | $0.55 \pm 0.05$ | Slope of linear region |
| Latitude | $L$ | $> 2.8$ stops | $D_{\min}+0.1$ to $D_{\max}-0.1$ |

**Rust Verification**:
```rust
assert!(hd_curve.gamma >= 0.50 && hd_curve.gamma <= 0.60);
assert!(hd_curve.d_max >= 2.8);
```

### 1.2 Reciprocity Failure
**Standard**: ISO 839
**Measurement Object**: Exposure time $t$ from 1/1000 s to 10 s, keeping $H = I \cdot t$ constant
**Pass Thresholds**:
- Density drift $< 0.15$ (1/1000 s to 1 s)
- Color shift $\Delta E_{00} < 3.0$ (Layer failure desynchronization at long exposures)

---

## Layer 2: Chemical Coupling (Dye Coupling)

### 2.1 Coupling Efficiency $\beta$
**Standard**: Kodak internal process specs (reverse engineerable)
**Measurement Object**: Dye density generated per unit silver density
**Formula**: $D_{\text{dye}} = \beta \cdot (D_{\text{silver}} - D_{\min})$
**Pass Thresholds**:
- Yellow layer $\beta_Y = 1.8 \pm 0.1$
- Magenta layer $\beta_M = 2.0 \pm 0.1$ (Higher extinction coefficient for Magenta dye)
- Cyan layer $\beta_C = 1.9 \pm 0.1$

### 2.2 Interlayer Interimage Effects (IIE)
**Standard**: ISO 4090
**Measurement Object**: Inhibition/Enhancement of lower layer development by upper layer exposure
**Test Chart**: Red/Green/Blue monochromatic wedges + Neutral gray wedge side-by-side
**Pass Thresholds**:
- Development inhibition rate $< 8\%$ (High density in upper layer causes density drop in lower layer)
- Edge Effect MTF 50% frequency $> 40$ lp/mm

---

## Layer 3: Optical Output

### 3.1 Status A Density (Print Viewing)
**Standard**: ISO 5-3
**Measurement Object**: RGB density measured through Wratten 106/92/88 filters
**Pass Thresholds**:
- Neutral Gray $R=G=B \pm 0.05$ (Status A)
- Orange Mask Base Color $D_R - D_B = 0.70 \pm 0.05$ (Typical Color Negative Mask)

### 3.2 Spectral Transmittance $T(\lambda)$
**Standard**: ISO 5-1
**Measurement Object**: $T(\lambda) = 10^{-A(\lambda)}$, where $A(\lambda) = \sum C_i \varepsilon_i(\lambda)$
**Pass Thresholds**:
- RMSE with real film transmittance spectra $< 0.02$ (400-700 nm)

---

## Layer 4: Colorimetric Accuracy
