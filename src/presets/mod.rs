//! Film stock presets organized by manufacturer

use crate::film::FilmStock;
use std::rc::Rc;

pub mod agfa;
pub mod fujifilm;
pub mod ilford;
pub mod kodak;
pub mod other;
pub mod polaroid;

// Re-export commonly used presets for convenience
pub use agfa::{VISTA_100, VISTA_200, VISTA_400};
pub use fujifilm::{ASTIA_100F, NEOPAN_100, PROVIA_100F, SUPERIA_200, SUPERIA_400, VELVIA_50};
pub use ilford::{
    DELTA_100_PROFESSIONAL, DELTA_400_PROFESSIONAL, FP4_PLUS_125, HP5_PLUS_400, PAN_F_PLUS_50,
    SFX_200,
};
pub use kodak::{
    KODAK_EKTACHROME_100VS, KODAK_EKTAR_100, KODAK_GOLD_200, KODAK_KODACHROME_25,
    KODAK_KODACHROME_64, KODAK_PLUS_X_125, KODAK_PORTRA_160, KODAK_PORTRA_400, KODAK_PORTRA_800,
    KODAK_TRI_X_400,
};
pub use other::STANDARD_DAYLIGHT;
pub use polaroid::POLAROID_SX70_COLOR;

/// Get all available film stock presets
pub fn get_all_stocks() -> Vec<Rc<FilmStock>> {
    let mut stocks = Vec::new();

    // Collect stocks from all manufacturers
    stocks.extend(kodak::get_stocks().into_iter().map(Rc::from));
    stocks.extend(fujifilm::get_stocks().into_iter().map(Rc::from));
    stocks.extend(ilford::get_stocks().into_iter().map(Rc::from));
    stocks.extend(agfa::get_stocks().into_iter().map(Rc::from));
    stocks.extend(polaroid::get_stocks().into_iter().map(Rc::from));
    stocks.extend(other::get_stocks().into_iter().map(Rc::from));

    stocks
}
