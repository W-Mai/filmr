//! Film stock presets organized by manufacturer

use crate::film::FilmStock;
use std::rc::Rc;

pub mod agfa;
pub mod fujifilm;
pub mod ilford;
pub mod kodak;
pub mod other;
pub mod polaroid;

// Re-export all presets for convenience
pub use agfa::*;
pub use fujifilm::*;
pub use ilford::*;
pub use kodak::*;
pub use other::*;
pub use polaroid::*;

/// Get all available film stock presets
pub fn get_all_stocks() -> Vec<Rc<FilmStock>> {
    vec![
        Rc::from(STANDARD_DAYLIGHT()),
        Rc::from(KODAK_TRI_X_400()),
        Rc::from(VELVIA_50()),
        Rc::from(HP5_PLUS_400()),
        Rc::from(KODAK_PORTRA_400()),
        Rc::from(KODAK_EKTAR_100()),
        Rc::from(KODAK_PORTRA_800()),
        Rc::from(DELTA_100_PROFESSIONAL()),
        Rc::from(SUPERIA_400()),
        Rc::from(VELVIA_50()),
        Rc::from(VISTA_100()),
        Rc::from(PROVIA_100F()),
        Rc::from(ASTIA_100F()),
        Rc::from(SUPERIA_400()),
        Rc::from(SUPERIA_200()),
        Rc::from(VISTA_200()),
        Rc::from(VISTA_400()),
        Rc::from(SUPERIA_200()),
        Rc::from(SUPERIA_400()),
        Rc::from(KODAK_TRI_X_400()),
        Rc::from(KODAK_EKTAR_100()),
        Rc::from(KODAK_PLUS_X_125()),
        Rc::from(FP4_PLUS_125()),
        Rc::from(DELTA_400_PROFESSIONAL()),
        Rc::from(PAN_F_PLUS_50()),
        Rc::from(SFX_200()),
        Rc::from(KODAK_PORTRA_160()),
        Rc::from(KODAK_GOLD_200()),
        Rc::from(KODAK_KODACHROME_25()),
        Rc::from(KODAK_KODACHROME_64()),
        Rc::from(KODAK_EKTACHROME_100VS()),
        Rc::from(NEOPAN_100()),
        Rc::from(POLAROID_SX70_COLOR()),
    ]
}
