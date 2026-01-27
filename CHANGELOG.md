# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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

## [0.1.0] - 2026-01-25

### 🎉 Initial Release

- Basic Film Simulation Engine (Physics-based).
- Support for 33 Film Stocks (Kodak, Fujifilm, Ilford, Polaroid).
- Spectral Sensitivity Simulation.
- Grain Simulation (RMS-based).
- Halation and Bloom effects.
- Basic GUI Demo with real-time preview.
