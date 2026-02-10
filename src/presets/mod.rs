//! Film stock presets organized by manufacturer

use crate::film::FilmStock;
use std::rc::Rc;

pub mod agfa;
pub mod fujifilm;
pub mod ilford;
pub mod kodak;
pub mod other;
pub mod polaroid;

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
