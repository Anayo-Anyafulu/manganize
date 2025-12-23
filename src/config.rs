use serde::{Deserialize, Serialize};

/// Quality preset for processing
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum QualityPreset {
    Fast,
    Balanced,
    Quality,
    Maximum,
}

impl QualityPreset {
    pub fn upscale_factor(&self) -> u32 {
        match self {
            QualityPreset::Fast => 1,
            QualityPreset::Balanced => 2,
            QualityPreset::Quality => 3,
            QualityPreset::Maximum => 4,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            QualityPreset::Fast => "Fast conversion (1x, basic detection)",
            QualityPreset::Balanced => "Balanced quality (2x, CV detection)",
            QualityPreset::Quality => "High quality (3x, enhanced processing)",
            QualityPreset::Maximum => "Maximum quality (4x, all features)",
        }
    }
}

impl std::str::FromStr for QualityPreset {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "fast" => Ok(QualityPreset::Fast),
            "balanced" => Ok(QualityPreset::Balanced),
            "quality" => Ok(QualityPreset::Quality),
            "maximum" | "max" => Ok(QualityPreset::Maximum),
            _ => Err(format!("Invalid preset: '{}'. Valid options: fast, balanced, quality, maximum", s)),
        }
    }
}

/// Configuration for manganize
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub general: GeneralConfig,
    
    #[serde(default)]
    pub detection: DetectionConfig,
    
    #[serde(default)]
    pub processing: ProcessingConfig,
    
    #[serde(default)]
    pub ai: AiConfig,
    
    #[serde(default)]
    pub output: OutputConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GeneralConfig {
    #[serde(default = "default_verbose")]
    pub verbose: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DetectionConfig {
    #[serde(default = "default_min_panel_size")]
    pub min_panel_size: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingConfig {
    #[serde(default = "default_aspect_ratio")]
    pub aspect_ratio: String,
    
    #[serde(default = "default_upscale_factor")]
    pub upscale_factor: u32,
    
    #[serde(default = "default_threads")]
    pub threads: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AiConfig {
    #[serde(default = "default_ollama_url")]
    pub ollama_url: String,
    
    #[serde(default = "default_ollama_model")]
    pub ollama_model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OutputConfig {
    #[serde(default = "default_output")]
    pub default_output: String,
}

// Default values
fn default_verbose() -> bool { false }
fn default_min_panel_size() -> u32 { 2500 }
fn default_aspect_ratio() -> String { "9:16".to_string() }
fn default_upscale_factor() -> u32 { 2 }
fn default_threads() -> usize { num_cpus::get() }
fn default_ollama_url() -> String { "http://localhost:11434".to_string() }
fn default_ollama_model() -> String { "llama3.2".to_string() }
fn default_output() -> String { "output.pdf".to_string() }

impl Default for GeneralConfig {
    fn default() -> Self {
        Self { verbose: default_verbose() }
    }
}

impl Default for DetectionConfig {
    fn default() -> Self {
        Self { min_panel_size: default_min_panel_size() }
    }
}

impl Default for ProcessingConfig {
    fn default() -> Self {
        Self {
            aspect_ratio: default_aspect_ratio(),
            upscale_factor: default_upscale_factor(),
            threads: default_threads(),
        }
    }
}

impl Default for AiConfig {
    fn default() -> Self {
        Self {
            ollama_url: default_ollama_url(),
            ollama_model: default_ollama_model(),
        }
    }
}

impl Default for OutputConfig {
    fn default() -> Self {
        Self { default_output: default_output() }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            detection: DetectionConfig::default(),
            processing: ProcessingConfig::default(),
            ai: AiConfig::default(),
            output: OutputConfig::default(),
        }
    }
}

impl Config {
    /// Load config from TOML file
    pub fn from_file(path: &std::path::Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save config to TOML file
    pub fn save_to_file(&self, path: &std::path::Path) -> anyhow::Result<()> {
        let content = toml::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Merge CLI args into config
    pub fn merge_with_args(&mut self, args: &crate::cli::Args) {
        if args.verbose {
            self.general.verbose = true;
        }
        
        if args.upscale_factor > 0 {
            self.processing.upscale_factor = args.upscale_factor;
        }
        
        // Apply quality preset
        if let Some(preset) = args.preset {
            self.processing.upscale_factor = preset.upscale_factor();
        }
    }
}
