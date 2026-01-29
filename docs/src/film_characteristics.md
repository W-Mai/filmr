# Film Characteristics

The "flavor" differences between films from different manufacturers are essentially reflected in **measurable parameters** determined by **chemical formulas** and **crystal structures**. Below are real parameters from authoritative technical data sheets.

---

## 1. KODAK TRI-X 400 (Classic News Film)

**Source**: Kodak Official Technical Data Sheet F-4017

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Sensitivity** | ISO 400/27° | Base exposure index |
| **RMS Granularity** | 17 (D-96 dev, 6 min) | Higher value means more visible grain |
| **Gamma (Contrast)** | 0.70 (Standard dev) | Medium contrast, good shadow detail retention |
| **Dmax** | 2.2-2.4 | Maximum black density |
| **Dmin** | 0.10-0.12 | Base + fog density |
| **Spectral Sensitivity** | Panchromatic, peak ~550nm | Response extends to ~690nm |
| **Resolution** | 100 lp/mm (1000:1 contrast) | Resolving power at high contrast |
| **Reciprocity Failure** | +1 stop compensation at 1s | Long exposure characteristic |

**Flavor Characteristics**: Wide exposure latitude (±2 stops still usable), rugged "breathing" grain, good shadow detail retention.

---

## 2. FUJIFILM Velvia 50 (Landscape Slide Film)

**Source**: Fujifilm Technical Data Sheet

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Sensitivity** | ISO 50/18° | Daylight base |
| **Gamma** | 1.2-1.4 (E-6 process) | Extremely high contrast, saturated colors |
| **Dmax** | 3.5-4.0 | Higher maximum density for slide film |
| **Dmin** | 0.15-0.20 | Transparent base density |
| **Granularity** | RMS 9 (Granularity Index) | Extremely fine grain |
| **Spectral Sensitivity** | Peak R=650nm, G=550nm, B=450nm | Strong red layer response, darkened blue sky |
| **Resolution** | 160 lp/mm | Ultra-high resolution, sharp details |
| **Reciprocity Failure** | +1/3 stop above 1/4000s | Short exposure characteristic |

**Flavor Characteristics**: High color saturation (especially red, orange), strong contrast, deeper blue sky rendering, fine grain but narrow exposure latitude (±0.5 stops).

---

## 3. ILFORD HP5 Plus (General Purpose B&W)

**Source**: ILFORD Official Data Sheet

| Parameter | Value | Description |
|-----------|-------|-------------|
| **Sensitivity** | ISO 400/27° | Base value, pushable to EI 3200 |
| **RMS Granularity** | 16 (D-96 dev) | Close to Tri-X, traditional cubic crystals |
| **Gamma** | 0.65-0.75 | Low-medium contrast, suitable for scanning |
| **Dmax** | 2.1-2.3 | Maximum density |
| **Dmin** | 0.08-0.10 | High base transparency |
| **Spectral Sensitivity** | Panchromatic, tested at 2850K Tungsten | Red response slightly lower than Tri-X |
| **Resolution** | 95 lp/mm | Slightly lower than Tri-X |
| **Push Characteristics** | Gamma rises to 0.9 at EI 1600 | Significant contrast increase when pushed |

**Flavor Characteristics**: Excellent push performance (usable at EI 3200), soft grain structure, smooth highlight roll-off, rich shadow gradations.

---

## 4. Mathematical Expression of Parameter Differences

These parameters directly determine the shape of the **H-D Curve**:

$$
D(E) = D_{\text{min}} + \frac{D_{\text{max}}-D_{\text{min}}}{1 + 10^{\gamma(\log E_0 - \log E)}}
$$

- **Tri-X**: Lower $E_0$, long toe, large latitude
- **Velvia**: Extremely high $D_{\text{max}}$, large $\gamma$, steep shoulder
- **HP5**: Lower $D_{\text{min}}$, $E_0$ shifts right when pushed

**Spectral Sensitivity Differences**:
$$ S(\lambda)_{\text{Velvia}} > S(\lambda)_{\text{Tri-X}} > S(\lambda)_{\text{HP5}} \quad (\lambda > 600\text{nm}) $$

---

## 5. Authoritative Verification Suggestions

1. **Official Data Sheets**: Kodak Alaris (kodakprofessional.com), Fujifilm, ILFORD (ilfordphoto.com)
2. **Third-party Testing**: Imatest software MTF analysis, X-Rite densitometer measurement of Dmax/Dmin
3. **Measurement Methods**: Verify Gamma using Stouffer step wedge + standard development process

These parameters are direct quantifications of film **chemical formulas** and **crystal technologies**, which are more reliable than subjective "flavor" descriptions.
