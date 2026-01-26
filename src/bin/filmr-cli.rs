use clap::{Parser, ValueEnum};
use filmr::presets;
use filmr::processor::{estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode};
use filmr::film::FilmStock;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Input image path
    #[arg(short, long)]
    input: PathBuf,

    /// Output image path
    #[arg(short, long)]
    output: PathBuf,

    /// Film preset to use
    #[arg(short, long, default_value = "kodak-portra-400")]
    preset: String,

    /// Exposure time override (default: auto-estimated)
    #[arg(short, long)]
    exposure: Option<f32>,

    /// Enable/Disable grain
    #[arg(short, long, default_value = "true")]
    grain: bool,

    /// Output mode: positive or negative
    #[arg(short = 'm', long, value_enum, default_value_t = CliOutputMode::Positive)]
    mode: CliOutputMode,

    /// White balance mode
    #[arg(short = 'w', long, value_enum, default_value_t = CliWhiteBalance::Auto)]
    wb: CliWhiteBalance,
}

#[derive(ValueEnum, Clone, Debug)]
enum CliOutputMode {
    Positive,
    Negative,
}

#[derive(ValueEnum, Clone, Debug)]
enum CliWhiteBalance {
    Auto,
    Off,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Loading image: {:?}", args.input);
    let img = image::open(&args.input)?.to_rgb8();

    let stock = find_preset(&args.preset).ok_or("Preset not found")?;
    println!("Using preset: {}", args.preset);

    let exposure = match args.exposure {
        Some(t) => t,
        None => {
            println!("Estimating exposure...");
            estimate_exposure_time(&img, &stock)
        }
    };
    println!("Exposure time: {:.4}s", exposure);

    let config = SimulationConfig {
        exposure_time: exposure,
        enable_grain: args.grain,
        output_mode: match args.mode {
            CliOutputMode::Positive => OutputMode::Positive,
            CliOutputMode::Negative => OutputMode::Negative,
        },
        white_balance_mode: match args.wb {
            CliWhiteBalance::Auto => WhiteBalanceMode::Auto,
            CliWhiteBalance::Off => WhiteBalanceMode::Off,
        },
        white_balance_strength: 1.0,
    };

    println!("Processing...");
    let start = Instant::now();
    let result = process_image(&img, &stock, &config);
    let duration = start.elapsed();
    println!("Done in {:.2?}", duration);

    println!("Saving to: {:?}", args.output);
    result.save(&args.output)?;

    Ok(())
}

fn find_preset(name: &str) -> Option<FilmStock> {
    let stocks = presets::get_all_stocks();
    let normalized_name = name.to_lowercase().replace("-", " ");
    
    for (stock_name, stock) in stocks {
        if stock_name.to_lowercase() == normalized_name || 
           stock_name.to_lowercase().replace(" ", "-") == name.to_lowercase() {
            return Some(stock);
        }
    }
    None
}
