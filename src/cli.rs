use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(name = "manganize")]
#[command(about = "Convert comics and manga to mobile-optimized PDF", long_about = None)]
#[command(version)]
pub struct Args {
    /// Path to PDF or directory of images
    #[arg(value_name = "INPUT")]
    pub input: Option<PathBuf>,

    /// Output PDF path
    #[arg(short, long, value_name = "FILE")]
    pub output: Option<PathBuf>,

    /// Target aspect ratio (width:height)
    #[arg(long)]
    pub aspect_ratio: Option<String>,

    /// Upscaling factor (1-4)
    #[arg(long)]
    pub upscale_factor: u32,

    /// Quality preset: fast, balanced, quality, maximum
    #[arg(short = 'q', long, value_name = "PRESET")]
    pub preset: Option<crate::config::QualityPreset>,

    /// Load configuration from TOML file
    #[arg(short = 'c', long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Save current settings to config file
    #[arg(long, value_name = "FILE")]
    pub save_config: Option<PathBuf>,

    /// Number of threads for parallel processing (0 = auto)
    #[arg(long, default_value = "0")]
    pub threads: usize,

    /// Ollama API URL
    #[arg(long)]
    pub ollama_url: Option<String>,

    /// Ollama model name
    #[arg(long)]
    pub ollama_model: Option<String>,

    /// Verbose output
    #[arg(short, long)]
    pub verbose: bool,

    /// Show progress bar
    #[arg(long, default_value = "true")]
    pub progress: bool,
}

impl Args {
    pub fn parse() -> Self {
        <Self as Parser>::parse()
    }

    /// Get final upscale factor (from preset or explicit value)
    pub fn get_upscale_factor(&self) -> u32 {
        if let Some(preset) = self.preset {
            preset.upscale_factor()
        } else if self.upscale_factor > 0 {
            self.upscale_factor
        } else {
            2 // default
        }
    }
}
