fn main() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(feature = "cli")]
    {
        filmr_app::cli::run()
    }
    #[cfg(not(feature = "cli"))]
    {
        eprintln!("Error: CLI feature is not enabled.");
        std::process::exit(1);
    }
}
