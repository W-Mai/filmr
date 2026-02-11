#[cfg(not(target_arch = "wasm32"))]
use filmr::film::FilmStockCollection;
#[cfg(not(target_arch = "wasm32"))]
use rfd::FileDialog;

use crate::ui::app::FilmrApp;

#[cfg(not(target_arch = "wasm32"))]
pub fn import_preset(app: &mut FilmrApp, changed: &mut bool) {
    if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).pick_file() {
        if let Ok(file) = std::fs::File::open(&path) {
            let reader = std::io::BufReader::new(file);
            if let Ok(collection) = serde_json::from_reader::<_, FilmStockCollection>(reader) {
                for (name, mut stock) in collection.stocks {
                    if stock.name.is_empty() {
                        stock.name = name;
                    }
                    app.stocks.push(std::rc::Rc::from(stock));
                }
                app.status_msg = "Loaded preset collection".to_string();
                *changed = true;
            } else if let Ok(mut stock) = filmr::FilmStock::load_from_file(&path) {
                let name = path.file_stem().unwrap().to_string_lossy().to_string();
                if stock.name.is_empty() {
                    stock.name = name.clone();
                }
                app.stocks.push(std::rc::Rc::from(stock));
                app.selected_stock_idx = app.stocks.len() - 1;
                app.load_preset_values();
                *changed = true;
                app.status_msg = format!("Loaded preset: {}", name);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn export_preset(app: &mut FilmrApp) {
    if let Some(path) = FileDialog::new().add_filter("JSON", &["json"]).save_file() {
        let mut stock = app.get_current_stock().as_ref().clone();
        stock.halation_strength = app.halation_strength;
        stock.halation_threshold = app.halation_threshold;
        stock.halation_sigma = app.halation_sigma;
        stock.grain_model.alpha = app.grain_alpha;
        stock.grain_model.sigma_read = app.grain_sigma;
        stock.grain_model.roughness = app.grain_roughness;
        stock.grain_model.blur_radius = app.grain_blur_radius;
        stock.r_curve.gamma *= app.gamma_boost;
        stock.g_curve.gamma *= app.gamma_boost;
        stock.b_curve.gamma *= app.gamma_boost;

        if stock.save_to_file(&path).is_ok() {
            app.status_msg = format!("Saved preset to {:?}", path);
        }
    }
}

pub fn create_custom_stock(app: &mut FilmrApp, ctx: &egui::Context) {
    use crate::ui::app::AppMode;

    let current_stock = app.get_current_stock().as_ref().clone();
    let base_name = app.stocks[app.selected_stock_idx].full_name();
    let clean_name = base_name.strip_prefix("Custom - ").unwrap_or(&base_name);
    let new_name = format!("Custom - {}", clean_name);
    let mut new_stock = current_stock;
    new_stock.name = new_name;
    new_stock.manufacturer = "".to_string();
    app.stocks.push(std::rc::Rc::from(new_stock.clone()));
    let new_idx = app.stocks.len() - 1;
    app.selected_stock_idx = new_idx;
    app.studio_stock = new_stock;
    app.studio_stock_idx = Some(new_idx);
    app.mode = AppMode::StockStudio;
    app.has_unsaved_changes = true;
    app.process_and_update_texture(ctx);
}
