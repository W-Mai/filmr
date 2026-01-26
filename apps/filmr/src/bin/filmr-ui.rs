fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "ui")]
    {
        match filmr_app::ui::run() {
            Ok(_) => Ok(()),
            Err(e) => Err(Box::new(e)),
        }
    }
    #[cfg(not(feature = "ui"))]
    {
        eprintln!("Error: UI feature is not enabled.");
        std::process::exit(1);
    }
}
