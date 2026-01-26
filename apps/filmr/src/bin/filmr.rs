use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    // Check if "ui" is explicitly requested
    // usage: filmr-xi ui
    if args.len() > 1 && args[1] == "ui" {
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
    } else {
        // Default to CLI
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
}
