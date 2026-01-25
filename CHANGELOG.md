# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-01-26

### 🚀 Features

- **Core**: Achieved 100% pass rate in industrial-grade quality verification (33/33 stocks).
- **Core**: Added `Paper Gamma` simulation (2.0 for Neg, 1.5 for Slide) to Positive output mode for realistic contrast restoration.
- **Core**: Optimized spectral fidelity checks to support Extended Red / IR sensitivity (up to 750nm).
- **GUI**: Implemented asynchronous image processing with spinner feedback to prevent UI freezing.
- **GUI**: Moved "Metrics Panel" toggle to the top-right corner for better UX.
- **GUI**: Added "Hold to Compare" feature for instant A/B testing.

### ♻️ Refactor

- **GUI**: Modularized `gui_demo` architecture into `panels/` (controls, metrics, central) and `app.rs`.
- **Core**: Refactored `verify_quality` tool to correctly handle B&W film validation (exempting color-based IIE/Skin checks).
- **Core**: Tuned `Fujifilm Astia 100F` and `Provia 400X` curves for better d_min/d_max compliance.

### 🐛 Fixes

- **Core**: Fixed Reciprocity Failure testing logic to use Linear Intensity instead of sRGB values.
- **Core**: Fixed "Channel Integrity" check for B&W films (panchromatic sensitivity is not leakage).
- **GUI**: Fixed main thread blocking by offloading heavy processing to background worker threads.

## [0.1.0] - 2026-01-25

### 🎉 Initial Release

- Basic Film Simulation Engine (Physics-based).
- Support for 33 Film Stocks (Kodak, Fujifilm, Ilford, Polaroid).
- Spectral Sensitivity Simulation.
- Grain Simulation (RMS-based).
- Halation and Bloom effects.
- Basic GUI Demo with real-time preview.
