# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.6.7] - 2026-02-11

### ♻️ Refactor

- **IO**: Separate EXIF metadata building from file I/O operations.
- **IO**: Unify image encoding and EXIF writing to memory (`Vec<u8>`) before file output.
- **IO**: Use `write_to_vec()` instead of `write_to_file()` for EXIF metadata to enable cross-platform support.
- **IO**: Remove platform-specific `#[cfg]` guards from `build_exif_metadata()` - now works on both desktop and WASM.
- **IO**: Consolidate file I/O to single point: `std::fs::write()` for desktop, `handle.write()` for WASM.
- **UI**: Split `controls.rs` into modular components (film_list, technical, simple).
- **UI**: Split `app.rs` into modular components (io, state, update).
- **GPU**: Split `gpu_pipelines.rs` into modular pipeline stages.
- **Presets**: Split into manufacturer-specific modules (kodak, fujifilm, ilford, vintage).
- **App**: Extract EXIF orientation utils to shared module.

### 💄 Style

- **UI**: Update window size to 1200x800 and rename title.

### 🔧 Fixes

- **Presets**: Fix imports after module refactoring.
- **Presets**: Fix `get_stocks()` function calls in all preset modules.
- **Examples**: Update preset function names after module split.

## [0.6.6] - 2026-02-07

### ✨ Features

- **Controls**: Improve film stock list row interaction with full row hover/click highlighting, better padding, and rounded thumbnail corners in Simple mode.

## [0.6.5] - 2026-02-07

### ✨ Features

- **App**: Preserve original EXIF metadata when saving images and add Filmr processing info (Software, ImageDescription, Copyright tags).

## [0.6.4] - 2026-02-07

### ✨ Features

- **App**: Improve save dialog with source-based default filename (`{source}_FILMR.jpg`) and JPEG as default format.

### 🐛 Fixes

- **App**: Fix `image::ImageOutputFormat` API change for image crate 0.25 compatibility.

## [0.6.3] - 2026-02-06

### ✨ Features

- **App**: Add EXIF orientation support for JPEG images. Images are now automatically rotated based on EXIF Orientation tag (supports all 8 orientations).

## [0.6.2] - 2026-02-06

### 🐛 Fixes

- **Controls**: Fix gamma boost (contrast) not applied to preset thumbnails. Thumbnail worker now receives the modified `FilmStock` with gamma boost instead of using the original preset.

## [0.6.1] - 2026-02-06

### ✨ Features

- **Settings**: Add optional histogram smoothing toggle (3-tap `[1,2,1]/4` weighted average) to reduce visual jitter in shadow regions.

## [0.6.0] - 2026-02-06

### ♻️ Refactoring

- **Core**: Extracted spectral matrix computation into `FilmStock::compute_spectral_matrix()`, eliminating duplicated spectral integration logic across pipeline and processor modules.

### ⚡️ Performance

- **Spectral**: Added SIMD (`f32x4`) optimization to `Spectrum` arithmetic operators (`Add`, `Mul`), reducing per-operation cost for spectral calculations.

### ✨ Features

- **GPU**: Enhanced develop shader with spectral/color dual matrix pipeline, shoulder softening compression, and logistic sigmoid H-D curve (replacing erf approximation) for better CPU/GPU consistency.
- **Grain**: Switched grain shader shadow noise model from inverse-distance to exponential decay (`exp(-2D)`), providing smoother shadow-to-midtone grain transitions.

### 🐛 Fixes

- **Metrics**: Fixed PSD slope calculation to use correct row-then-column 2D FFT instead of flattened 1D FFT.

## [0.5.9] - 2026-02-04

### ♻️ Refactoring

- **Core**: Refactored `Spectrum` struct to remove `Copy` trait, forcing explicit data flow and improving performance by avoiding expensive implicit copies.
- **Core**: Improved numerical stability in `new_blackbody` and `integrate_product` functions within `spectral.rs`, ensuring physical consistency and preventing potential overflows.
- **Core**: Replaced implicit amplitude scaling with `new_gaussian_normalized` to guarantee energy conservation in spectral modeling regardless of bandwidth.

### ✨ Features

- **Grain**: Implemented resolution-dependent grain scaling. Grain blur radius and noise amplitude now scale automatically with image resolution (reference 2K), ensuring consistent visual graininess across different image sizes.

## [0.5.8] - 2026-02-04

### ♻️ Refactoring

- **UI**: Replaced internal `cus_component` module with `egui-uix` external crate for better code reuse.
- **Core**: Refactored `processor.rs` to extract GPU pipeline execution and CPU fallback logic, improving modularity and maintainability.
- **Core**: Implemented `OnceLock` caching for GPU pipelines in `gpu_pipelines.rs` to avoid redundant initialization and improve "Hot Run" performance.

## [0.5.7] - 2026-02-04

### ⚡️ Performance

- **Bench**: Added `benchmark_sop` example tool for standardized performance testing (24MP image, Cold vs Hot runs).
- **Core**: Optimized `processor.rs` instrumentation using `tracing` spans for better profiling visibility.

## [0.5.6] - 2026-02-03

### 🚀 Features

- **GPU**: Implemented GPU acceleration for **Light Leak** and **Halation** stages using compute shaders.

## [0.5.5] - 2026-02-03

### 🚀 Features

- **WASM**: Implemented global GPU context management for WASM workers.

### 🐛 Fixes

- **WASM**: Disabled GPU context on WASM temporarily to fix build issues.

## [0.5.4] - 2026-02-03

### 🚀 Features

- **GPU**: Implemented **Linearization** compute shader for GPU pipeline entry.

### 🐛 Fixes

- **GPU**: Fixed buffer usage validation errors for map read operations by implementing proper staging buffers.

### ⚡️ Performance

- **GPU**: Optimized data transfer using storage buffers instead of uniform buffers.

## [0.5.3] - 2026-02-03

### ⚡️ Performance

- **Core**: Implemented SIMD optimizations for **Gaussian Blur**, **Spectral** calculations, and **Halation** effect on CPU path.

## [0.5.2] - 2026-02-02

### 🚀 Features

- **WASM**: Implemented multi-threaded image processing using `rayon` and `wasm-bindgen-rayon` for significantly improved performance on the web.
- **WASM**: Added dedicated `ComputeBridge` and Web Worker infrastructure to handle heavy computations off the main UI thread.
- **WASM**: Integrated `console_log` for unified logging in the browser console.

### 🐛 Fixes

- **WASM**: Fixed `hist_rgb` serialization issue by implementing `serde-big-array` wrapper, ensuring correct histogram data transfer between worker and UI.
- **WASM**: Resolved "Parking not supported" panic by enabling `parking_lot/nightly` and `wasm-bindgen-rayon/no-bundler` features.
- **WASM**: Fixed `worker.js` module loading errors by patching import paths and removing problematic `modulepreload` links.
- **CI**: Fixed GitHub Actions workflow for WASM builds by switching to `nightly` toolchain and adding `rust-src` component (required for `build-std` and atomics).
- **Scripts**: Enhanced `patch_dist.py` robustness with better regex matching and environment variable support.

## [0.5.1] - 2026-01-30

### 🚀 Features

- **UI**: Added "Save" and "Back" buttons in Studio Mode for improved workflow.
- **UI**: Optimized main layout by removing the top panel and relocating the settings button for a cleaner interface.
- **UI**: Improved positioning of the UX mode toggle.

### 🐛 Fixes

- **Metrics**: Fixed logic for retrieving and displaying film metrics.

### ♻️ Refactoring

- **Core**: Refactored `FilmStock` struct to embed `manufacturer` and `name` fields directly, simplifying the `presets` module and removing redundant tuple wrappers.
- **Core**: Optimized `FilmStock` usage to prefer references and moves over cloning, improving performance and reducing unnecessary allocations.


## [0.5.0] - 2026-01-30

### 🚀 Features

- **UI**: Introduced **Simple** and **Professional** UX modes. Simple mode focuses on quick adjustments (Brightness, Contrast, Warmth, Intensity), while Professional mode offers full physics-based control.
- **UI**: Added **Split-Screen Comparison** view in the central panel for side-by-side before/after comparison.
- **UI**: Implemented async **Preset Thumbnails** in the controls panel for visual preview of film stocks.
- **Processor**: Added `warmth` and `saturation` parameters to the simulation engine.
- **UI**: Added persistency for UX mode preference in `config.json`.
- **UI**: Optimized top bar layout with direct mode toggles.

### 🐛 Fixes

- **Metrics**: Fixed incorrect metrics display logic and optimized metrics panel visibility for different modes.
- **UI**: Fixed control panel layout issues and improved visual hierarchy with better spacing and grouping.

### ⚠ Breaking Changes

- **Core**: Refactored `FilmStock` struct to embed `manufacturer` and `name` fields directly, removing the need for external name management. Updated `get_all_stocks` to return `Vec<FilmStock>` instead of tuples.

## [0.4.0] - 2026-01-29

### ⚠ Breaking Changes

- **Core**: Refactored `ReciprocityFailure` model to be a struct with `beta` parameter, removing the `description` field. This restores `Copy` trait for `FilmStock`.
- **Core**: Removed references to internal documentation IDs ("tec n") from public API comments.

### 🚀 Features

- **Core**: Added standard color negative spectral response model (`new_color_negative_standard`).
- **Ops**: Implemented structured logging with `tracing` crate, replacing `println!` debugging.
- **Docs**: Established `mdBook` knowledge base structure in `docs/`.
- **Docs**: Added comprehensive Rustdoc documentation to public API (`FilmStock`, `PipelineStage`, etc.).

### ⚡️ Performance

- **Bench**: Added `criterion` benchmarks for image processing (1080p).

### 🐛 Fixes

- **Tests**: Resolved Clippy warnings and unused variables in test suites.

## [0.3.9] - 2026-01-27

### ⚡️ Performance

- **UI**: Offloaded image decoding, analysis, and resize operations to a background worker thread to prevent UI blocking during file load.

## [0.3.8] - 2026-01-27

### 🚀 Features

- **UI**: Auto-process images immediately upon loading.
- **UI**: Reset export status when a new image is loaded.
- **Performance**: Conditional scaling for preview images (only resize if > 2048px).

## [0.3.7] - 2026-01-27

### 🚀 Features

- **UI**: Added "Settings" menu item to File menu.

### 🐛 Fixes

- **UI**: Fixed spinner positioning to be a centered overlay.

## [0.3.6] - 2026-01-27

### 🚀 Features

- **UI**: Implemented settings window and persistent configuration management.
- **UI**: Enhanced preview logic with initial, interaction, and develop states.
- **UI**: Improved Stock Studio with "Edit" capability for imported stocks.
- **UI**: Added semi-transparent spinner overlay with dynamic status text.

## [0.3.5] - 2026-01-27

### 🚀 Features

- **Core**: Enhanced Light Leak simulation with organic/plasma shapes and rotation support.
- **UI**: Added controls for Light Leak configuration (Shape, Rotation, Intensity).

### 🐛 Fixes

- **UI**: Fixed portrait image blur by increasing preview texture resolution.

## [0.3.4] - 2026-01-27

### 🚀 Features

- **Core/UI**: Implemented `ConfigManager` for persistent settings.
- **Core**: Added support for `FilmStockCollection` and loading custom presets from JSON.
- **UI**: Added ability to import and auto-load custom film collections.

## [0.3.3] - 2026-01-27

### 💄 Style

- **UI**: Changed default font to `ark-pixel` for better legibility.

## [0.3.2] - 2026-01-27

### 🚀 Features

- **UI**: Added "Stock Studio" for custom film creation and editing.
- **UI**: Implemented "Exit Dialog" to warn about unsaved changes.
- **UI**: Added "Status Bar" for displaying application state.
- **UI**: Enabled "Sync" of studio edits to the stock list.
- **UI**: Added "Create Custom Stock" from current selection.

### 🐛 Fixes

- **UI**: Restored drag-and-drop functionality.

## [0.3.0] - 2026-01-26

### 🚀 Features

- **Core**: Added Light Leak simulation with configurable parameters.
- **CLI**: Introduced `filmr-cli` command line tool.
- **Core**: Implemented advanced RMS grain roughness simulation.
- **Core**: Added Serde serialization for film types and preset management (Save/Load/Export/Import).
- **Architecture**: Restructured project into a workspace with core library and unified app.

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

## [0.1.5] - 2026-01-25

### 🚀 Features

- **Docs**: Added README and LICENSE.
- **Docs**: Added detailed GUI demo run instructions.
- **Demo**: Added metrics info and original image display to GUI demo.
- **Demo**: Implemented large image preview and develop/save workflow.
- **Demo**: Moved metrics to right side panel and improved visualization with `egui_plot`.
- **Demo**: Optimized preview size and dynamic scaling for metrics plots.

### 🐛 Fixes

- **Demo**: Fixed histogram visualization issues and GLCM display.
- **Demo**: Fixed chart scaling and legend issues.

## [0.1.4] - 2026-01-24

### 🚀 Features

- **Quality**: Implemented comprehensive 7-layer verification (neutral axis, channel integrity, spectral fidelity, etc.).
- **Quality**: Added automated color fidelity verification.
- **Quality**: Implemented consolidated diagnosis chart and report generation.
- **Metrics**: Integrated film metrics (Dynamic Range, MTF, etc.) into diagnosis.

## [0.1.3] - 2026-01-23

### 🚀 Features

- **Pipeline**: Implemented base fog anchoring and D65 illuminant handling.
- **Exposure**: Improved auto-exposure time estimation logic.

## [0.1.2] - 2026-01-22

### 🚀 Features

- **Spectral**: Implemented wavelength-based simulation and relative sensitivity factors.
- **Spectral**: Tuned sensitivity curves to fix yellow tint and red deficiency.
- **Spectral**: Implemented gray-world auto white balance blend.
- **Presets**: Added 30+ new film simulation presets.
- **GUI**: Added preset selector and detailed halation controls.
- **Grain**: Implemented monochrome grain for B&W films.

### 🐛 Fixes

- **Core**: Fixed panic on non-square images.
- **Presets**: Configured B&W films to produce correct grayscale output.

## [0.1.1] - 2026-01-22

### 🚀 Features

- **Core**: Initial implementation of physical models (layer occlusion, etc.).
- **Refactor**: Extracted film presets to `presets.rs` and made `FilmStock` customizable.

## [0.1.0] - 2026-01-22

### 🎉 Initial Release

- Basic Film Simulation Engine (Physics-based).
- Support for initial set of Film Stocks.
- Spectral Sensitivity Simulation foundation.
- Grain Simulation (RMS-based).
- Halation and Bloom effects.
- Basic GUI Demo with real-time preview.
