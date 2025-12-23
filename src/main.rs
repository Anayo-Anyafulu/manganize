use anyhow::{Context, Result};
use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

mod cli;
mod config;
mod input;
mod panel_detector;
mod reading_unit_grouper;
mod image_processor;
mod pdf_generator;
mod types;

use cli::Args;
use config::Config;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    let args = Args::parse();
    
    // Handle save-config option
    if let Some(save_path) = &args.save_config {
        let config = build_config_from_args(&args)?;
        config.save_to_file(save_path)?;
        println!("✅ Configuration saved to: {}", save_path.display());
        return Ok(());
    }
    
    // Load or create config
    let mut config = if let Some(config_path) = &args.config {
        Config::from_file(config_path)
            .context(format!("Failed to load config from {:?}", config_path))?
    } else {
        Config::default()
    };
    
    // Merge CLI args into config
    config.merge_with_args(&args);
    
    // Validate input
    let input = args.input.ok_or_else(|| {
        anyhow::anyhow!("Input file or directory required. Use --help for usage.")
    })?;
    
    let output = args.output
        .unwrap_or_else(|| PathBuf::from(&config.output.default_output));
    
    // Set number of threads
    let threads = if args.threads == 0 {
        config.processing.threads
    } else {
        args.threads
    };
    rayon::ThreadPoolBuilder::new()
        .num_threads(threads)
        .build_global()
        .ok();
    
    // Display banner
    println!();
    println!("{}", style("🎨 Manganize - Comic to Mobile PDF Converter").bold().cyan());
    println!("{}", style("━".repeat(50)).dim());
    
    if let Some(preset) = args.preset {
        println!("📊 Quality: {} ({})", 
            style(format!("{:?}", preset)).yellow(),
            style(preset.description()).dim()
        );
    }
    
    println!("📁 Input:  {}", style(input.display()).green());
    println!("📄 Output: {}", style(output.display()).green());
    println!("🔢 Threads: {}", style(threads).cyan());
    println!("⚡ Upscale: {}x", style(config.processing.upscale_factor).cyan());
    println!();
    
    // Step 1: Load input
    let pb = create_spinner("Loading input...");
    let pages = input::load_comic(&input)
        .context("Failed to load input file")?;
    pb.finish_with_message(format!("✓ Loaded {} pages", style(pages.len()).green()));
    
    // Step 2: Detect panels
    let pb = create_spinner("Detecting panels...");
    let panels = panel_detector::detect_panels(&pages, args.verbose)
        .context("Failed to detect panels")?;
    pb.finish_with_message(format!("✓ Detected {} panels", style(panels.len()).green()));
    
    // Step 3: Group into reading units
    let pb = create_spinner("Grouping panels (using AI)...");
    let ollama_url = args.ollama_url
        .unwrap_or_else(|| config.ai.ollama_url.clone());
    let ollama_model = args.ollama_model
        .unwrap_or_else(|| config.ai.ollama_model.clone());
    
    let reading_units = reading_unit_grouper::group_panels(
        panels,
        &ollama_url,
        &ollama_model,
        args.verbose
    ).await
    .context("Failed to group panels into reading units")?;
    pb.finish_with_message(format!("✓ Created {} reading units", style(reading_units.len()).green()));
    
    // Step 4: Process images
    let pb = if args.progress {
        let pb = ProgressBar::new(reading_units.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{msg} [{bar:40.cyan/blue}] {pos}/{len} ({percent}%)")
                .unwrap()
                .progress_chars("█▓░")
        );
        pb.set_message("⚡ Processing images");
        Some(pb)
    } else {
        println!("⚡ Processing images...");
        None
    };
    
    let aspect_ratio = args.aspect_ratio
        .unwrap_or_else(|| config.processing.aspect_ratio.clone());
    
    let processed_units = image_processor::process_reading_units(
        reading_units,
        aspect_ratio,
        config.processing.upscale_factor,
        args.verbose
    ).context("Failed to process images")?;
    
    if let Some(pb) = pb {
        pb.finish_with_message(format!("✓ Processed {} reading units", style(processed_units.len()).green()));
    } else {
        println!("✓ Processed {} reading units", processed_units.len());
    }
    
    // Step 5: Generate PDF
    let pb = create_spinner("Generating PDF...");
    pdf_generator::create_pdf(&processed_units, &output)
        .context("Failed to generate PDF")?;
    pb.finish_with_message(format!("✓ PDF created: {}", style(output.display()).green()));
    
    println!();
    println!("{}", style("🎉 Conversion complete!").bold().green());
    println!("{}", style("━".repeat(50)).dim());
    println!("📱 Your mobile-optimized comic is ready: {}", style(output.display()).bold());
    println!();
    
    Ok(())
}

fn create_spinner(msg: &str) -> ProgressBar {
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.cyan} {msg}")
            .unwrap()
    );
    pb.set_message(msg.to_string());
    pb.enable_steady_tick(std::time::Duration::from_millis(100));
    pb
}

fn build_config_from_args(args: &Args) -> Result<Config> {
    let mut config = Config::default();
    config.merge_with_args(args);
    Ok(config)
}
