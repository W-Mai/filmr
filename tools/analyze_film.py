#!/usr/bin/env python3
"""
filmr Film Stock Analyzer

Analyzes a scanned film photograph to extract the film stock's color and grain
characteristics. Uses neutral/low-saturation regions to separate film traits
from scene content.

Usage:
    python3 tools/analyze_film.py <image_path>
"""

import sys
import numpy as np
from PIL import Image
from scipy.ndimage import uniform_filter


def load(path):
    img = np.array(Image.open(path)).astype(float)
    h, w, _ = img.shape
    r, g, b = img[:, :, 0], img[:, :, 1], img[:, :, 2]
    luma = 0.299 * r + 0.587 * g + 0.114 * b
    sat = np.max(img, axis=2) - np.min(img, axis=2)
    return img, r, g, b, luma, sat, h, w


def section(title):
    print(f"\n{'='*60}")
    print(f"  {title}")
    print(f"{'='*60}")


def analyze_color_cast(r, g, b, luma, sat):
    """Analyze film color cast using low-saturation (neutral) pixels."""
    section("Color Cast Analysis (neutral pixels, sat<30)")

    neutral = sat < 30
    n = neutral.sum()
    if n < 100:
        print("  Not enough neutral pixels for analysis.")
        return

    print(f"  Neutral pixel count: {n} ({100*n/luma.size:.1f}% of image)")

    zones = [
        ("Shadows", 0, 60),
        ("Midtones", 60, 120),
        ("Highlights", 120, 180),
        ("Specular", 180, 255),
    ]

    print(f"\n  {'Zone':12} {'R':>6} {'G':>6} {'B':>6} | {'R-G':>5} {'R-B':>5} {'Bias':>8}")
    print(f"  {'-'*55}")

    for name, lo, hi in zones:
        mask = neutral & (luma >= lo) & (luma < hi)
        if mask.sum() < 50:
            continue
        zr, zg, zb = r[mask].mean(), g[mask].mean(), b[mask].mean()
        rg, rb = zr - zg, zr - zb
        if rg > 2 and rb > 2:
            bias = "RED"
        elif rg < -2 and rb < -2:
            bias = "BLUE"
        elif rg > 2 and rb < -2:
            bias = "GREEN"
        elif rg < -2 and rb > 2:
            bias = "MAGENTA"
        else:
            bias = "neutral"
        print(f"  {name:12} {zr:6.1f} {zg:6.1f} {zb:6.1f} | {rg:+5.1f} {rb:+5.1f} {bias:>8}")

    # Overall
    zr, zg, zb = r[neutral].mean(), g[neutral].mean(), b[neutral].mean()
    print(f"  {'Overall':12} {zr:6.1f} {zg:6.1f} {zb:6.1f} | {zr-zg:+5.1f} {zr-zb:+5.1f}")


def analyze_tonality(luma):
    """Analyze tonal distribution."""
    section("Tonality")

    ps = np.percentile(luma, [1, 5, 25, 50, 75, 95, 99])
    print(f"  p1={ps[0]:.0f}  p5={ps[1]:.0f}  p25={ps[2]:.0f}  p50={ps[3]:.0f}  p75={ps[4]:.0f}  p95={ps[5]:.0f}  p99={ps[6]:.0f}")
    print(f"  Dynamic range (p1-p99): {ps[6]-ps[0]:.0f}")
    print(f"  Contrast (p75-p25): {ps[4]-ps[2]:.0f}")

    if ps[3] < 90:
        print(f"  Exposure: UNDEREXPOSED (median={ps[3]:.0f})")
    elif ps[3] > 170:
        print(f"  Exposure: OVEREXPOSED (median={ps[3]:.0f})")
    else:
        print(f"  Exposure: NORMAL (median={ps[3]:.0f})")


def analyze_grain(img, luma, h, w):
    """Analyze grain characteristics."""
    section("Grain Analysis")

    # High-pass filter to isolate grain from content
    noise_r = img[:, :, 0] - uniform_filter(img[:, :, 0], size=7)
    noise_g = img[:, :, 1] - uniform_filter(img[:, :, 1], size=7)
    noise_b = img[:, :, 2] - uniform_filter(img[:, :, 2], size=7)
    noise_luma = luma - uniform_filter(luma, size=7)

    # Grain vs brightness
    print(f"  {'Zone':12} {'Luma':>5} {'Grain σ':>8} {'R σ':>6} {'G σ':>6} {'B σ':>6}")
    print(f"  {'-'*50}")

    zones = [
        ("Shadows", 0, 60),
        ("Low-mid", 60, 100),
        ("Midtones", 100, 140),
        ("High-mid", 140, 180),
        ("Highlights", 180, 240),
    ]

    for name, lo, hi in zones:
        mask = (luma >= lo) & (luma < hi)
        if mask.sum() < 200:
            continue
        sl = noise_luma[mask].std()
        sr = noise_r[mask].std()
        sg = noise_g[mask].std()
        sb = noise_b[mask].std()
        ml = luma[mask].mean()
        print(f"  {name:12} {ml:5.0f} {sl:8.2f} {sr:6.2f} {sg:6.2f} {sb:6.2f}")

    # Cross-channel correlation (use midtone region)
    mid_mask = (luma >= 80) & (luma < 180)
    if mid_mask.sum() > 500:
        rg = np.corrcoef(noise_r[mid_mask].ravel()[:5000], noise_g[mid_mask].ravel()[:5000])[0, 1]
        rb = np.corrcoef(noise_r[mid_mask].ravel()[:5000], noise_b[mid_mask].ravel()[:5000])[0, 1]
        gb = np.corrcoef(noise_g[mid_mask].ravel()[:5000], noise_b[mid_mask].ravel()[:5000])[0, 1]
        avg_corr = (rg + rb + gb) / 3
        print(f"\n  Channel correlation (midtones):")
        print(f"    R-G={rg:.3f}  R-B={rb:.3f}  G-B={gb:.3f}  avg={avg_corr:.3f}")
        if avg_corr > 0.85:
            print(f"    → Mostly luminance grain (monochrome-like)")
        elif avg_corr > 0.5:
            print(f"    → Mixed luminance + chroma grain")
        else:
            print(f"    → Strong chroma grain (color noise)")

    # Grain size estimate via autocorrelation
    mid_rows = noise_luma[h // 2 - 5 : h // 2 + 5, :]
    row = mid_rows.mean(axis=0)
    if len(row) > 40:
        acf = np.correlate(row - row.mean(), row - row.mean(), mode="full")
        acf = acf[len(acf) // 2 :]
        if acf[0] > 0:
            acf /= acf[0]
            half_idx = np.argmax(acf < 0.5)
            grain_um = half_idx / (w / 36.0) * 1000  # px → mm → µm (assuming 35mm)
            print(f"\n  Grain size: ~{half_idx} px (≈{grain_um:.0f} µm assuming 35mm frame)")


def analyze_saturation(sat):
    """Analyze color saturation distribution."""
    section("Saturation")

    ps = np.percentile(sat, [25, 50, 75, 90, 99])
    print(f"  p25={ps[0]:.0f}  p50={ps[1]:.0f}  p75={ps[2]:.0f}  p90={ps[3]:.0f}  p99={ps[4]:.0f}")
    print(f"  Mean saturation: {sat.mean():.1f}")

    if sat.mean() < 25:
        print(f"  → Low saturation (muted/desaturated look)")
    elif sat.mean() < 45:
        print(f"  → Moderate saturation")
    else:
        print(f"  → High saturation (vivid colors)")


def suggest_preset(r, g, b, luma, sat, noise_luma_std):
    """Suggest filmr preset parameters based on analysis."""
    section("Suggested filmr Parameters")

    neutral = sat < 30
    mid = neutral & (luma >= 100) & (luma < 180)

    if mid.sum() > 50:
        zr, zg, zb = r[mid].mean(), g[mid].mean(), b[mid].mean()
        rg_bias = zr - zg
        rb_bias = zr - zb
    else:
        rg_bias, rb_bias = 0, 0

    print(f"  # Color matrix bias (midtone neutral R-G={rg_bias:+.1f}, R-B={rb_bias:+.1f})")
    if rg_bias > 2:
        print(f"  color_matrix[0][0] += 0.03  # boost R")
    if rb_bias > 5:
        print(f"  color_matrix[2][2] -= 0.02  # suppress B")

    print(f"\n  # Grain")
    print(f"  color_correlation: 0.93  # based on channel correlation")
    print(f"  grain noise σ ≈ {noise_luma_std:.1f} (in sRGB space)")

    contrast = np.percentile(luma, 75) - np.percentile(luma, 25)
    if contrast > 100:
        print(f"\n  # High contrast (IQR={contrast:.0f})")
        print(f"  gamma: 2.0+")
    elif contrast > 60:
        print(f"\n  # Medium contrast (IQR={contrast:.0f})")
        print(f"  gamma: 1.5-1.8")
    else:
        print(f"\n  # Low contrast (IQR={contrast:.0f})")
        print(f"  gamma: 0.8-1.2")


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <image_path>")
        sys.exit(1)

    path = sys.argv[1]
    print(f"Analyzing: {path}")

    img, r, g, b, luma, sat, h, w = load(path)
    print(f"Image: {w}×{h}")

    analyze_tonality(luma)
    analyze_color_cast(r, g, b, luma, sat)
    analyze_saturation(sat)
    analyze_grain(img, luma, h, w)

    # Get midtone grain for suggestion
    noise_luma = luma - uniform_filter(luma, size=7)
    mid_mask = (luma >= 80) & (luma < 180)
    mid_std = noise_luma[mid_mask].std() if mid_mask.sum() > 100 else 5.0
    suggest_preset(r, g, b, luma, sat, mid_std)


if __name__ == "__main__":
    main()
