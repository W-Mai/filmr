use crate::exif_utils::{apply_exif_orientation, read_exif_orientation};
use clap::{Parser, ValueEnum};
use filmr::film::{FilmStock, FilmStockCollection};
use filmr::presets;
use filmr::processor::{
    estimate_exposure_time, process_image, OutputMode, SimulationConfig, WhiteBalanceMode,
};
use std::io::BufReader;
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

    /// Export the selected preset to a JSON file
    #[arg(long)]
    export_preset: Option<PathBuf>,

    /// Load a custom preset from a JSON file (overrides --preset)
    #[arg(long)]
    load_preset: Option<PathBuf>,

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

pub fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    println!("Loading image: {:?}", args.input);

    // Read EXIF orientation first
    let orientation = std::fs::File::open(&args.input)
        .ok()
        .map(|f| {
            let mut reader = BufReader::new(f);
            read_exif_orientation(&mut reader)
        })
        .unwrap_or(1);

    // Load and apply orientation
    let img = {
        let raw = image::open(&args.input)?;
        apply_exif_orientation(raw, orientation).to_rgb8()
    };

    let stock = if let Some(path) = &args.load_preset {
        println!("Loading custom preset from: {:?}", path);
        // Try to load as collection first
        let file = std::fs::File::open(path)?;
        let reader = std::io::BufReader::new(file);

        if let Ok(collection) = serde_json::from_reader::<_, FilmStockCollection>(reader) {
            println!("Detected preset collection.");
            // Try to find the preset specified by --preset argument
            if let Some(s) = collection.stocks.get(&args.preset) {
                println!("Using preset '{}' from collection.", args.preset);
                std::rc::Rc::from(s.clone())
            } else {
                // If not found, list available keys
                let keys: Vec<_> = collection.stocks.keys().collect();
                return Err(format!(
                    "Preset '{}' not found in collection. Available presets: {:?}",
                    args.preset, keys
                )
                .into());
            }
        } else {
            // Fallback to single stock
            std::rc::Rc::from(FilmStock::load_from_file(path)?)
        }
    } else {
        find_preset(&args.preset).ok_or("Preset not found")?
    };

    if let Some(export_path) = &args.export_preset {
        println!("Exporting preset to: {:?}", export_path);
        stock.save_to_file(export_path)?;
    }

    println!(
        "Using preset: {}",
        if args.load_preset.is_some() {
            "Custom"
        } else {
            &args.preset
        }
    );

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
        ..Default::default()
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

fn find_preset(name: &str) -> Option<std::rc::Rc<FilmStock>> {
    let stocks = presets::get_all_stocks();
    let normalized_name = name.to_lowercase().replace("-", " ");

    for stock in stocks {
        let stock_name = stock.full_name();
        if stock_name.to_lowercase() == normalized_name
            || stock_name.to_lowercase().replace(" ", "-") == name.to_lowercase()
        {
            return Some(stock);
        }
    }
    None
}
