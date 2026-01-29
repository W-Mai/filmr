# Dynamic Range and Highlight Roll-off

A common observation in film simulation is that **digital dynamic range numbers win, but film highlight behavior wins on "character"**. This is not a "resolution" issue, but a **dimensionality superiority of chemical response**.

---

## 1. Dynamic Range: The Definition Trap

| Metric | High-end CMOS (Sony A7R V) | Kodak Portra 400 |
|--------|---------------------------|------------------|
| **Engineering DR** | 15.3 stops (Linear RAW) | 13.2 stops (Density 2.8) |
| **Effective DR** | 13.5 stops (Noise ≤ 1 ADU) | **18+ stops (Printable Highlights)** |

**Key**: CMOS hard clips at 14 bit (Full Well Capacity), whereas film dye continues to stack after $D_{\max}=2.8$—it just transitions from **linear growth to logarithmic decay**. Scanners might clip, but optical enlargers can retrieve this, giving film its "highlight retention" ability.

---

## 2. Three Stages of Chemical Highlight Roll-off

### Stage 1: Linear Region (0–0.8 Density)
Silver halide grains respond linearly according to Beer-Lambert law: $D = \gamma \cdot \log_{10}(H)$. This looks like standard LOG encoding.

### Stage 2: Shoulder Softening (0.8–1.8 Density)
**Chemical Mechanisms**:
1. Grain surface development centers saturate → Electron trapping efficiency $\eta$ drops, following **Mott-Gurney Space Charge Limit**: $J \propto V^2/d^3$
2. Developer oxidation product (QDI) decomposes at high pH → Coupling rate $k_{\text{coupling}}$ decays
3. **Interlayer Inhibition**: High density in upper layers releases Bromide ions (Br⁻) to lower layers, inhibiting their development (ISO 4090 IIE effect)

Mathematically:
$$
D(H) = \gamma \log_{10}(H) \cdot \left(1 - \frac{H}{H_{\text{sat}}}\right)^{\alpha}
$$
Where $\alpha \approx 0.65$ (Kodak Patent US4508812A). This is the **core of highlight softening**.

### Stage 3: Toe Clipping (>1.8 Density)
Dye molecules stack to the **Förster Resonance Energy Transfer** distance (< 2 nm), quenching energy → Density growth approaches $D_{\max}$ asymptotically.

---

## 3. Why Digital Post-Processing Falls Short

### 3.1 Dimensionality Gap
CMOS highlights have only **16384 discrete levels (14 bit)**. Once clipped, data is gone.
Film highlights still have **31-dimensional spectral** changes, just with slower density growth. Post-processing can only **interpolate**, not **extrapolate** real spectra.

### 3.2 Irreversibility of Non-linear Coupling
Photoshop's "Highlight Compression" is $y = \log(x + \epsilon)$ or $y = x^{0.8}$, applied **independently per channel**.
Film involves **Yellow + Magenta + Cyan dye densities inhibiting each other**. Mathematically, it's a **ternary non-linear coupled system**:

$$
\begin{cases}
C_Y = f_Y(H_B, H_G, H_R) \\
C_M = f_M(H_G, H_R, H_Y) \\
C_C = f_C(H_R, H_Y, H_G)
\end{cases}
$$

Standard tools don't model "upper layer exposure poisoning lower layer".

### 3.3 Noise Texture Differences
CMOS highlight noise is **Poisson + Readout Noise**. Film is **Granularity following Wiener Spectrum**: $G(f) = \frac{a}{f^2 + b}$, which **decreases** as density increases.
Digital noise addition is just "adding salt", lacking density-dependent grain size distribution.

---

## 4. Hard Simulation: Adding "Chemical Constraints"

### 4.1 Shoulder Softening Model
Apply a **space charge limit** after Exposure → Density mapping:

```rust
fn shoulder_softening(density: f32, shoulder_point: f32) -> f32 {
    if density > shoulder_point {
        let excess = density - shoulder_point;
        density - excess * excess / (shoulder_point + excess)
    } else {
        density
    }
}
```
Where `shoulder_point = 0.8` (ISO 5800 standard point).

### 4.2 Interlayer Inhibition Matrix
```rust
// Interlayer effect matrix from ISO 4090 measurements
// Row i = Real Dye i / Theoretical Dye i
let interlayer: nalgebra::Matrix3<f32> = nalgebra::Matrix3::new(
    1.00, -0.08, -0.03, // Yellow inhibited by Magenta/Cyan
    -0.05, 1.00, -0.07, // Magenta inhibited by Yellow/Cyan
    -0.02, -0.06, 1.00, // Cyan inhibited by Yellow/Magenta
);

let dyes_real = interlayer * dyes_theoretical;
```

### 4.3 Dye Self-Absorption Correction
```rust
// Dye self-absorption causes spectral shift at high densities
fn dye_self_absorb(c: f32, t: &mut [f32]) {
    // c: dye concentration, t: transmittance spectrum
    for wl in 0..31 {
        // Beer's law deviation of 1-3% when density > 1.5
        let correction = 1.0 + (c - 1.5) * 0.02;
        t[wl] *= correction.clamp(0.97, 1.03);
    }
}
```
