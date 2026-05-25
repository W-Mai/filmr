//! Snapshot structures for undo/redo history.

use filmr::{light_leak::LightLeakConfig, FilmStyle, OutputMode, WhiteBalanceMode};
use serde::{Deserialize, Serialize};

/// Snapshot of all develop-mode adjustments. Excludes UI state (zoom, selected
/// tab) and image data (texture handles, depth maps) — only user-edited values.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopSnapshot {
    pub exposure_time: f32,
    pub gamma_boost: f32,
    pub warmth: f32,
    pub saturation: f32,

    pub halation_strength: f32,
    pub halation_threshold: f32,
    pub halation_sigma: f32,

    pub grain_alpha: f32,
    pub grain_sigma: f32,
    pub grain_roughness: f32,
    pub grain_blur_radius: f32,
    pub motion_blur_amount: f32,
    pub motion_blur_seed: u64,
    pub object_motion_amount: f32,
    pub auto_levels: bool,
    pub dof_amount: f32,
    pub dof_focus: f32,
    pub dof_swirl: f32,
    pub rotational_blur_amount: f32,

    pub light_leak_config: LightLeakConfig,

    pub selected_stock_idx: usize,
    pub film_style: FilmStyle,

    pub output_mode: OutputMode,
    pub white_balance_mode: WhiteBalanceMode,
    pub white_balance_strength: f32,
}
