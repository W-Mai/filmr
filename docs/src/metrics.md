# Metrics and Diagnosis

| Metric/Method | Category | Calculation/Core Principle | Simulation Usage/Diagnostic Value |
|---------------|----------|--------------------------|-----------------------------------|
| **Histogram** | Basic Stats | Pixel value binning, 1D distribution | Overall exposure distribution, dynamic range check |
| **Quantiles (p50/p90/p99)** | Basic Stats | Percentiles of CDF | Locating median color casts (e.g., `rg_mid p50`) |
| **Mean/Median** | Basic Stats | Weighted average or p50 | Overall exposure level baseline |
| **Standard Deviation** | Basic Stats | Sqrt of variance | Contrast quantification (film usually lower than digital) |
| **Skewness** | Basic Stats | 3rd central moment | Asymmetry, negative skew = more highlight detail (film trait) |
| **Kurtosis** | Basic Stats | 4th central moment | Tail thickness, high kurtosis = clipped blacks/whites |
| **Entropy** | Basic Stats | `-Σp(x)log p(x)` | Information density, grain increases local entropy |
| **Clipped Pixel Ratio** | Basic Stats | Ratio of extreme values | Proportion of dead blacks and blown highlights |
| **Dynamic Range** | Basic Stats | p99/p1 or p995/p005 | Effective latitude quantification |
| **RgMid/BgMid** | Color Science | R/G, B/G ratio at mid-gray | White balance baseline, deviation from 1.0 = cast |
| **Lab Space Stats** | Color Science | L*a*b* channel mean/var | L* lightness latitude, a*/b* shows cast direction |
| **CCT & Tint** | Color Science | Inverted gray point estimation | White balance calibration verification |
| **Saturation Dist** | Color Science | HSV S-channel skewness | Asymmetric saturation distribution of film |
| **Delta E 2000** | Color Science | CIEDE2000 formula | Difference between simulation and target scan |
| **PSD (Power Spectral Density)** | Frequency/Texture | Radial avg of FFT | Grain frequency distribution (`1/f^β` noise) |
| **MTF** | Frequency/Texture | Contrast vs Spatial Frequency | Sharpness quantification, edge retention |
| **Laplacian Variance** | Frequency/Texture | 2nd derivative stats | Fast sharpness estimation |
| **LBP (Local Binary Patterns)** | Frequency/Texture | Local texture coding | Grain structure difference quantification |
| **SSIM/MS-SSIM** | Perceptual/Struct | Structural Similarity | Structure fidelity (Sim vs Ref) |
| **LPIPS** | Perceptual/Struct | VGG feature distance | Perceptual distance, better than L2 |
| **CID (Contrast Index)** | Perceptual/Struct | CIE-based contrast index | Contrast difference quantification |
| **Dye Density Curve** | Film Specific | Density vs log Exposure | H&D curve fitting (R²/RMSE) |
| **Mask Density** | Film Specific | Orange mask a*/b* offset | Negative-to-positive mask correction verification |
| **Reciprocity Curve** | Film Specific | Time-Density non-linearity | Short/Long exposure simulation accuracy |
| **Grain Autocorrelation** | Film Specific | Spatial correlation of noise | Film grain is not pure random noise |
| **HSV Histogram** | Color Space | 1D/Multi-D binning | Hue shifts, saturation asymmetry |
| **Lab Histogram** | Color Space | L*a*b* channel stats | Lightness latitude, color cast direction |
| **YCbCr Histogram** | Color Space | Luma/Chroma separation | Chroma range check (film gamut is often narrower) |
| **RGB Joint Hist (3D)** | Color Space | R×G×B cube binning | Color mapping linearity, neutral axis shift |
| **2D Joint Hist** | Spatial Relation | Joint prob (e.g., R-G) | Channel correlation, neutral axis diagonal check |
| **GLCM** | Spatial Relation | Co-occurrence probability | Grain texture quantification, uniformity |
| **Tile Histogram** | Spatial Relation | Block-wise stats | Vignetting, uneven development detection |
| **Power Spectrum Hist** | Frequency | 2D FFT radial binning | `Energy vs Frequency`, grain feature extraction |
| **Wavelet Hist** | Frequency | Multi-scale coeff stats | Multi-layer grain structure capture |
| **Gradient Hist** | Gradient/Edge | Edge direction stats | Silver halide crystal anisotropy detection |
| **Laplacian Hist** | Gradient/Edge | 2nd derivative stats | Sharpness/Softness measurement |
| **Ratio Hist (R/G)** | Ratio/Log Space | Channel ratio binning | Illumination-robust cast detection |
| **Log Exposure Hist** | Ratio/Log Space | Log-space binning | Direct fitting of H&D curve toe/shoulder |
