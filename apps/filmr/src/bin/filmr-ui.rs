fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(all(feature = "ui", not(target_arch = "wasm32")))]
    {
        match filmr_app::ui::run() {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
    #[cfg(all(feature = "ui", target_arch = "wasm32"))]
    {
        // On WASM, the entry point is lib::start, not main.
        // But if trunk builds this bin, we need to make sure main compiles.
        // Actually, we want trunk to build the library entry point.
        Ok(())
    }
    #[cfg(not(feature = "ui"))]
    {
        eprintln!("Error: UI feature is not enabled.");
        std::process::exit(1);
    }
}
