# Full-Spectrum Simulation Engine

## Overview

filmr 0.7.0 introduces a physics-based full-spectrum simulation engine that propagates light wavelength-by-wavelength through the film's multi-layer structure. This replaces the 3×3 matrix approximation used in Fast mode with a physically accurate model.

## Dual Simulation Modes

| Mode | Method | Use Case |
|------|--------|----------|
| **Fast** | 3×3 spectral matrix | Real-time preview |
| **Accurate** | Per-wavelength per-layer propagation | Final develop (Develop button) |

## Physical Model

### Light Propagation

Light enters the film from the top and traverses each layer sequentially:

1. **Fresnel reflection** at each interface: $R = \left(\frac{n_1 - n_2}{n_1 + n_2}\right)^2$
2. **Beer-Lambert absorption** within each layer: $I_{out} = I_{in} \cdot e^{-\alpha \cdot d}$
3. **Scattering loss**: modeled as additional attenuation coefficient
4. **Emulsion exposure**: absorbed energy (not scattered) is recorded per channel

### Backward Pass (Halation)

After traversing all layers, light reflects off the film base (Fresnel at base/air interface) and propagates back up through the stack. This physically models halation — the reddish glow around highlights caused by base reflection.

### Layer Structure

A typical colour negative film stack:

```
Incident light →
  [Overcoat]           n=1.50, transparent
  [Blue Emulsion]      n=1.53, absorbs ~450nm → Yellow dye
  [Yellow Filter]      n=1.52, blocks blue from lower layers
  [Green Emulsion]     n=1.53, absorbs ~545nm → Magenta dye
  [Interlayer]         n=1.50, spacer
  [Red Emulsion]       n=1.53, absorbs ~640nm → Cyan dye
  [Anti-Halation]      n=1.50, absorbs residual light
  [Base (PET)]         n=1.65, 125µm substrate
```

Each layer has:
- **Thickness** (µm)
- **Refractive index** (for Fresnel calculation)
- **Spectral absorption** (81 bins, 380–780nm, 5nm steps)
- **Scattering coefficient** (Mie/Rayleigh in emulsion)

### Interlayer Interimage Effect

During development, one layer's chemical byproducts inhibit adjacent layers (DIR coupler effect). This is modeled as a 3×3 inhibition matrix applied after H-D curve density calculation:

$$D_i' = D_i + \sum_j \text{inhibition}[i][j] \cdot D_j$$

Off-diagonal values are negative (suppression), improving colour separation.

## Spectral Data

- **CIE 1931 2° Standard Observer** (x̄, ȳ, z̄) — ISO 11664-1
- **CIE Standard Illuminant D65** — ISO 11664-2
- **sRGB camera sensitivities** derived from XYZ CMF × XYZ→sRGB matrix, D65-balanced

All data: 380–780nm, 5nm steps, 81 bins.

## Pipeline (Accurate Mode)

```
sRGB input
  → Linearize
  → Light Leak (additive)
  → Halation (threshold + blur)
  → Per-pixel spectral propagation:
      uplift(R,G,B) → spectrum × D65 → propagate(layer_stack)
      → integrate → normalize → × exposure_time
  → Scattering spatial diffusion (Gaussian blur from layer scatter)
  → White balance + warmth
  → log₁₀ → H-D curves + colour matrix
  → Interlayer inhibition
  → MTF blur
  → Grain
  → Density → Transmission → sRGB output
```

## Verification

69 tests verify the engine:
- 8 physical property tests (linearity, energy conservation, layer order, etc.)
- 6 strict physics tests (exact Beer-Lambert, Fresnel, scattering, energy budget)
- 17 per-stage model correctness tests
- 4 end-to-end integration tests
- 1 Fast/Accurate consistency test
