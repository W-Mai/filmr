# Rendering Pipeline

> Complete technical reference for filmr's physics-based film simulation.
> Every formula is verified against source code.

## Pipeline Overview

```
┌─────────────────────────────────────────────────────────┐
│  Input: sRGB u8 image                                   │
├─────────────────────────────────────────────────────────┤
│  Stage 0: Linearize          sRGB → Linear f32          │
│  Stage 1: Light Leak + Halation                         │
│  Stage 2: Full-Spectrum Develop                          │
│    ├ Pass 1: Per-pixel spectral propagation              │
│    ├ Pass 2: Scattering diffusion                        │
│    ├ Pass 2.5: White balance + warmth                    │
│    └ Pass 3: H-D curves + color matrix + inhibition      │
│  Stage 3: MTF blur                                       │
│  Stage 4: Grain                                          │
│  Stage 5: Output conversion   Density → sRGB u8         │
├─────────────────────────────────────────────────────────┤
│  Output: sRGB u8 image                                  │
└─────────────────────────────────────────────────────────┘
```

---

## Stage 0: Linearize

Digital images are stored in sRGB gamma encoding. The first step converts each pixel to linear light, which represents actual physical irradiance.

\\[
\text{linear} = \begin{cases}
\dfrac{v}{12.92} & v \le 0.04045 \\\\[6pt]
\left(\dfrac{v + 0.055}{1.055}\right)^{2.4} & v > 0.04045
\end{cases}
\\]

A 256-entry lookup table is precomputed for speed.

---

## Stage 1: Light Leak + Halation

### Halation

In real film, light passes through the emulsion layers, reflects off the film base, and scatters back into the emulsion. This creates a warm glow around bright highlights.

1. Compute luminance: \\( L = 0.2126R + 0.7152G + 0.0722B \\)
2. Extract highlights above threshold \\( T \\)
3. Gaussian blur with \\( \sigma = W \times \text{halation\_sigma} \\)
4. Blend back: \\( \text{pixel} \mathrel{+}= \text{blurred} \times \text{tint} \times \text{strength} \\)

The tint is typically warm orange `[1.0, 0.6, 0.3]`, matching the reddish glow seen in real film scans.

---

## Stage 2: Full-Spectrum Develop

This is the core of the simulation. Light is propagated wavelength-by-wavelength through the film's physical layer structure — modelling how real photons interact with silver halide crystals, dye layers, and the film base.

### The Film Layer Stack

A colour negative film is a sandwich of thin layers:

| Layer | Purpose | Typical thickness |
|-------|---------|:-:|
| Overcoat | Scratch protection | 1 µm |
| **Blue Emulsion** | Captures blue light → Yellow dye | 3–6 µm |
| **Yellow Filter** | Blocks blue from reaching lower layers | 1 µm |
| **Green Emulsion** | Captures green light → Magenta dye | 3–5 µm |
| Interlayer | Spacer | 1 µm |
| **Red Emulsion** | Captures red light → Cyan dye | 3–5 µm |
| Anti-Halation | Absorbs residual light | 2 µm |
| Base (PET) | Transparent substrate | 125 µm |

Each layer has:
- **Refractive index** \\( n \\) (~1.50–1.65)
- **Spectral absorption** \\( \alpha(\lambda) \\) — 81 values from 380–780 nm
- **Scattering coefficient** \\( s \\) — Mie/Rayleigh scattering in emulsion
- **Thickness** \\( d \\) in micrometers

### Precomputation

Before processing pixels, the engine precomputes per-layer coefficients for all 81 wavelength bins:

**Fresnel reflection** at each interface (how much light bounces back when entering a new medium):

\\[
R = \left(\frac{n_1 - n_2}{n_1 + n_2}\right)^2, \quad t = 1 - R
\\]

**Beer-Lambert transmission** (how much light survives passing through a layer):

\\[
\tau_i = \exp\bigl(-(\alpha_i + s) \cdot d\bigr)
\\]

**Absorption deposit** (fraction of light captured by an emulsion layer):

\\[
\delta_i = (1 - \tau_i) \times \frac{\alpha_i}{\alpha_i + s}
\\]

This separates absorption (which exposes the film) from scattering (which just removes light from the beam).

### Pass 1: Per-Pixel Spectral Propagation

#### Spectral Reconstruction

Each pixel's RGB values are expanded into a full 81-wavelength spectrum using precomputed sRGB spectral sensitivities and the D65 daylight illuminant:

\\[
P(\lambda_i) = R \cdot m_{R,i} + G \cdot m_{G,i} + B \cdot m_{B,i}
\\]

where \\( m_{ch,i} = S_{ch}(\lambda_i) \times D_{65}(\lambda_i) \\).

#### Forward Pass (Light Entering the Film)

For each layer, top to bottom:

1. **Fresnel**: \\( P_i \mathrel{\times}= t \\) — some light reflects at the interface
2. **Absorb** (emulsion only): \\( E_{ch,i} \mathrel{+}= P_i \times \delta_i \\) — film captures energy
3. **Attenuate**: \\( P_i \mathrel{\times}= \tau_i \\) — remaining light continues downward

What happens to different colours:

| | Blue (450 nm) | Green (545 nm) | Red (640 nm) |
|---|:---:|:---:|:---:|
| Blue Emulsion | **absorbed** | passes | passes |
| Yellow Filter | **blocked** | passes | passes |
| Green Emulsion | — | **absorbed** | passes |
| Red Emulsion | — | — | **absorbed** |

#### Backward Pass (Base Reflection)

Light that reaches the base partially reflects back:

\\[
P_{\text{back},i} = P_{\text{residual},i} \times \left(\frac{n_{\text{base}} - 1}{n_{\text{base}} + 1}\right)^2
\\]

This reflected light travels back up through all layers, depositing additional exposure in each emulsion. This is the physical origin of halation — the red emulsion (closest to the base) receives the most reflected light.

#### Integration

The per-wavelength exposure is integrated into three scalar values using the trapezoidal rule:

\\[
E_{ch} = \left[\sum_{i=1}^{79} e_i + \frac{1}{2}(e_0 + e_{80})\right] \times 5\text{ nm}
\\]

Finally, exposure is scaled by a normalization factor and the user's exposure time setting.

### Pass 2: Scattering Diffusion

Scattering within emulsion layers causes light to spread laterally, slightly blurring the image:

\\[
\sigma_{\text{px}} = \frac{\sum_l d_l \cdot s_l}{36000} \times W
\\]

In practice this is sub-pixel for most film stocks and has negligible visual effect.

### Pass 2.5: White Balance

Optionally equalizes the average colour of the image:

\\[
\text{gain}_{ch} = 1 + \left(\frac{\bar{L}}{\bar{C}_{ch}} - 1\right) \times \text{strength}
\\]

Warmth shifts the red/blue balance: \\( R \mathrel{\times}= (1 + w \cdot 0.1) \\), \\( B \mathrel{\times}= (1 - w \cdot 0.1) \\).

### Pass 3: H-D Curves + Color Matrix + Inhibition

#### The Characteristic Curve

The Hurter-Driffield (H-D) curve is the fundamental relationship between exposure and density in photographic film. It's the S-shaped curve that gives film its distinctive tonal response.

\\[
D = D_{\min} + (D_{\max} - D_{\min}) \cdot \frac{1}{1 + e^{-k \cdot x}}
\\]

where:

\\[
x = \log_{10} E - \log_{10} E_0, \quad k = \frac{4\gamma}{D_{\max} - D_{\min}}
\\]

| Parameter | Meaning | Typical range |
|-----------|---------|:---:|
| \\( D_{\min} \\) | Base fog density | 0.04–0.15 |
| \\( D_{\max} \\) | Maximum density | 2.5–3.3 |
| \\( \gamma \\) | Contrast (slope at midpoint) | 0.6–2.2 |
| \\( E_0 \\) | Speed point | varies |

- **Toe** (low exposure): gentle roll-off, shadow detail
- **Linear region**: proportional response, midtones
- **Shoulder** (high exposure): compression, highlight roll-off

#### Shoulder Softening

At very high densities, silver halide crystals saturate:

\\[
D > D_s: \quad D' = D - \frac{(D - D_s)^2}{D_s + (D - D_s)}
\\]

#### Color Matrix

Models inter-layer dye coupling. Each channel's net density is mixed:

\\[
\begin{pmatrix} D_R' \\\\ D_G' \\\\ D_B' \end{pmatrix} =
\mathbf{M} \begin{pmatrix} D_R - D_{\min,R} \\\\ D_G - D_{\min,G} \\\\ D_B - D_{\min,B} \end{pmatrix} +
\begin{pmatrix} D_{\min,R} \\\\ D_{\min,G} \\\\ D_{\min,B} \end{pmatrix}
\\]

#### Interlayer Inhibition

During development, chemical byproducts from one layer suppress development in adjacent layers (DIR coupler effect). This enhances colour separation.

The inhibition operates on density **deviation** from the mean, so neutral grays are unaffected:

\\[
\bar{D} = \frac{D_R + D_G + D_B}{3}, \quad \Delta D_i = D_i - \bar{D}
\\]

\\[
D_i^{\,\text{final}} = D_i + \sum_j \text{inh}_{i,j} \cdot \Delta D_j
\\]

Off-diagonal values are negative (suppression), increasing the spread between channels.

---

## Stage 3: MTF Blur

Every film has a finite resolving power. This is modelled as a Gaussian blur:

\\[
\sigma = \frac{0.5}{\text{resolution\_lp\_mm}} \times \frac{W}{36}
\\]

where \\( W \\) is image width in pixels and 36 mm is the standard 35mm film width.

---

## Stage 4: Grain

Film grain arises from the random distribution of silver halide crystals. The noise variance depends on density:

\\[
\sigma^2 = \bigl(\alpha \cdot D^{1.5} + \sigma_{\text{read}}^2 + \sigma_{\text{shot}}^2\bigr) \times \bigl(1 + r \cdot \sin(\pi D)\bigr)
\\]

- \\( \alpha \cdot D^{1.5} \\): grain increases with density (more developed crystals)
- \\( \sigma_{\text{read}}^2 \\): base fog noise
- \\( \sigma_{\text{shot}}^2 = s \cdot e^{-2D} \\): photon shot noise (strongest in shadows)
- Roughness \\( r \\): frequency modulation in midtones

Two grain layers are blended: fine crystals everywhere, plus coarse clumps that appear in highlights (silver halide aggregation at high density).

---

## Stage 5: Output Conversion

Converts the density image to a viewable photograph.

### Density → Transmission

\\[
T = 10^{-(D - D_{\min})}
\\]

High density = low transmission = dark on the negative.

### Dye Self-Absorption

At high densities, Beer's Law deviates slightly:

\\[
D > 1.5: \quad T' = T \times \text{clamp}\bigl(1 + 0.02(D - 1.5),\; 0.97,\; 1.03\bigr)
\\]

### Print Simulation

The negative is "printed" onto paper with its own gamma:

\\[
n = \frac{T_{\max} - T}{T_{\max} - T_{\min}}, \quad \text{output} = n^{\gamma_{\text{paper}}}
\\]

| Film type | Paper gamma |
|-----------|:---:|
| Colour negative | 2.0 |
| Colour slide | 1.5 |

### sRGB Encoding

\\[
\text{sRGB} = \begin{cases}
12.92 \cdot v & v \le 0.0031308 \\\\[4pt]
1.055 \cdot v^{1/2.4} - 0.055 & v > 0.0031308
\end{cases}
\\]

---

## Spectral Data

All spectral computations use standard reference data:

- **CIE 1931 2° Standard Observer** — colour matching functions \\( \bar{x}, \bar{y}, \bar{z} \\)
- **CIE Standard Illuminant D65** — daylight spectrum
- **sRGB primaries** — derived from XYZ CMF × IEC 61966-2-1 matrix

Wavelength range: 380–780 nm, 5 nm steps, 81 bins.

---

## Performance

| Resolution | Time |
|:---:|:---:|
| 256² | 6.5 ms |
| 512² | 20 ms |
| 1024² | 63 ms |

Key optimization: layer coefficients (Fresnel, Beer-Lambert, deposit) are precomputed once per frame, eliminating ~1782 `exp()` calls per pixel.

---

## Verification

72 tests ensure physical correctness:

| Category | Count | What's verified |
|----------|:---:|---------|
| Physical properties | 8 | Energy conservation, linearity, layer ordering |
| Exact physics | 6 | Beer-Lambert, Fresnel, scattering vs absorption |
| Inhibition | 3 | Neutral unaffected, colour separation enhanced |
| Per-stage | 17 | Each pipeline stage independently |
| End-to-end | 4 | Gray gradient, 6-colour hue card |
| Histogram | 2 | Full RGB distribution comparison |
