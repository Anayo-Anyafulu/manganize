# 🎨 Manganize

**Convert comics and manga into mobile-optimized PDFs for seamless vertical reading**

Manganize is a hybrid system that intelligently transforms traditional comic/manga files into mobile-friendly PDF documents. Using computer vision for panel detection and AI-powered narrative flow analysis, it creates reading units optimized for 9:16 vertical scrolling on phones.

## 💡 Why I Made This

This project is inspired by TikTok-style comic edits that reframe wide comic pages into vertically readable panels for mobile devices.

> **Reference:** [TikTok — Comic Edits](https://vm.tiktok.com/ZMDNuvvHY/)

Comics are usually drawn for big pages, not phones, so reading them on mobile feels zoomed out and uncomfortable. I saw TikTok videos where people cropped comic panels into vertical slides so you can read them one by one, and it made everything much clearer.

That inspired me to build a tool that:
- **Automatically detects** comic panels using AI/CV
- **Intelligently crops and groups** them
- **Upscales** low-quality images
- **Outputs a phone-friendly PDF**

The result is a comic that’s much easier to read on mobile without constantly zooming in and out. This tool automates that manual editing process while preserving the artistic intent.

## ✨ Features

- 📖 **Multi-format input**: Supports PDF files and image directories (JPEG, PNG, WEBP)
- 🔍 **Smart panel detection**: Computer vision-based contour detection
- 🤖 **AI-powered grouping**: Uses Ollama LLM to group panels into logical reading units
- ⚡ **High-performance processing**: Rust-based image pipeline with:
  - Multi-panel merging (vertical stacking, grid layouts)
  - 2× upscaling with Lanczos3 algorithm
  - Light enhancement (brightness, contrast)
  - Smart cropping to 9:16 aspect ratio
- 📱 **Mobile-optimized output**: Single PDF with smooth vertical scrolling

## 🚀 Quick Start

### Prerequisites

1. **Rust** (1.70+): Install from [rustup.rs](https://rustup.rs/)
2. **Ollama**: Install from [ollama.ai](https://ollama.ai/)
   ```bash
   # Start Ollama and pull a model
   ollama serve
   ollama pull llama3.2
   ```
3. **PDFium** (for PDF input support):
   ```bash
   # Linux
   sudo apt-get install libpdfium-dev
   
   # macOS
   brew install pdfium
   ```

### Installation

```bash
# Clone or navigate to the project
cd manganize

# Build release binary
cargo build --release

# (Optional) Install globally
cargo install --path .
```

### Usage

```bash
# Basic conversion
manganize input.pdf -o output.pdf

# Convert directory of images
manganize ./manga-chapter-1/ -o chapter1-mobile.pdf

# Use quality presets (fast, balanced, quality, maximum)
manganize input.pdf --preset quality

# Load settings from config file
manganize input.pdf --config manganize.toml

# Save current settings to config
manganize input.pdf --preset maximum --save-config my-settings.toml

# Advanced usage
manganize input.pdf \
  --preset quality \
  --threads 8 \
  --ollama-model llama3.2 \
  --verbose
```

### Command-Line Options

```
manganize [OPTIONS] <INPUT>

Arguments:
  <INPUT>  Path to PDF or directory of images

Options:
  -o, --output <FILE>           Output PDF path
  -q, --preset <PRESET>         Quality preset: fast, balanced, quality, maximum
  -c, --config <FILE>           Load configuration from TOML file
  --save-config <FILE>          Save current settings to config file
  --aspect-ratio <RATIO>        Target aspect ratio [default: 9:16]
  --upscale-factor <FACTOR>     Upscaling factor 1-4 [default: 2]
  --threads <NUM>               Number of threads (0 = auto) [default: 0]
  --ollama-url <URL>            Ollama API URL [default: http://localhost:11434]
  --ollama-model <MODEL>        Ollama model name [default: llama3.2]
  -v, --verbose                 Verbose output
  --progress                    Show progress bar [default: true]
  -h, --help                    Print help
  -V, --version                 Print version
```

### Quality Presets

| Preset | Upscale | Speed | Quality | Best For |
|--------|---------|-------|---------|----------|
| `fast` | 1× | ⚡⚡⚡ | ⭐ | Quick previews |
| `balanced` | 2× | ⚡⚡ | ⭐⭐⭐ | Most comics (default) |
| `quality` | 3× | ⚡ | ⭐⭐⭐⭐ | High-res displays |
| `maximum` | 4× | 🐌 | ⭐⭐⭐⭐⭐ | Archival quality |

### Configuration Files

Create a `manganize.toml` file to save your preferences:

```toml
[general]
verbose = false

[processing]
aspect_ratio = "9:16"
upscale_factor = 3
threads = 8

[ai]
ollama_url = "http://localhost:11434"
ollama_model = "llama3.2"

[output]
default_output = "mobile.pdf"
```

Then use it:
```bash
manganize comic.pdf --config manganize.toml
```

## 🏗️ Architecture

```
Input (PDF/Images)
    ↓
[Input Handler] ──→ Extract pages/load images
    ↓
[Panel Detector] ──→ CV-based contour detection → Bounding boxes
    ↓
[Reading Unit Grouper] ──→ Ollama LLM analysis → Grouped panels
    ↓
[Image Processor] ──→ Merge + Crop + Upscale + Enhance
    ↓
[PDF Generator] ──→ One page per reading unit
    ↓
Output (Mobile PDF)
```

## 🧩 Components

### 1. Input Handler (`input.rs`)
- Loads PDF files using `pdfium-render`
- Loads image directories (JPEG, PNG, WEBP)
- Converts to internal `Page` representation

### 2. Panel Detector (`panel_detector.rs`)
- Applies grayscale conversion and thresholding
- Detects contours to find panel boundaries
- Filters noise and full-page backgrounds
- Sorts panels by reading order (top→bottom, left→right)

### 3. Reading Unit Grouper (`reading_unit_grouper.rs`)
- Sends panel metadata to Ollama LLM
- AI analyzes narrative flow and visual relationships
- Returns grouped panels with layout strategies
- Fallback to individual panels if AI fails

### 4. Image Processor (`image_processor.rs`)
- **Merging**: Combines panels (vertical stack, grid)
- **Smart cropping**: Adjusts to 9:16 aspect ratio
- **Upscaling**: 2× quality enhancement with Lanczos3
- **Enhancement**: Brightness and contrast adjustments

### 5. PDF Generator (`pdf_generator.rs`)
- Creates PDF using `printpdf`
- One page per reading unit
- Optimized for mobile dimensions (72 DPI)

## 📊 Example Workflow

**Input**: `comic.pdf` (20 pages, traditional landscape layout)

1. **Panel Detection**: Detects 45 panels across 20 pages
2. **AI Grouping**: Creates 18 reading units (1-3 panels each)
3. **Image Processing**: 
   - Merges multi-panel units vertically
   - Crops to 1080×1920 (9:16)
   - Upscales to 2160×3840
   - Enhances lighting
4. **PDF Output**: `output.pdf` (18 pages, vertical scroll)

**Result**: Smooth mobile reading experience! 📱✨

## 🛠️ Development

### Run Tests
```bash
cargo test
```

### Build with Logging
```bash
RUST_LOG=debug cargo run -- input.pdf -v
```

### Project Structure
```
manganize/
├── Cargo.toml
├── README.md
├── src/
│   ├── main.rs                 # CLI orchestration
│   ├── cli.rs                  # Argument parsing
│   ├── types.rs                # Shared data structures
│   ├── input.rs                # PDF/image loading
│   ├── panel_detector.rs       # CV panel detection
│   ├── reading_unit_grouper.rs # Ollama LLM integration
│   ├── image_processor.rs      # Image pipeline
│   └── pdf_generator.rs        # PDF creation
└── examples/                   # Sample comics
```

## 🎯 Roadmap

- [x] MVP with CV panel detection
- [x] Ollama LLM integration
- [x] Image processing pipeline
- [x] PDF generation
- [ ] ONNX model integration for AI panel detection
- [ ] Face detection for smart cropping
- [ ] OCR text extraction for better AI context
- [ ] Parallel processing for large files
- [ ] GUI interface
- [ ] Batch processing mode

## 🐛 Troubleshooting

**"Failed to bind to PDFium library"**
- Install PDFium development libraries (see Prerequisites)
- Alternatively, use image directory input instead of PDF

**"Ollama request failed"**
- Ensure Ollama is running: `ollama serve`
- Check model is downloaded: `ollama pull llama3.2`
- Verify URL with `--ollama-url http://localhost:11434`

**Low-quality output**
- Increase upscale factor: `--upscale-factor 3`
- Use higher resolution input images
- Check source image quality

## 📝 License

MIT License - feel free to use and modify!

## 🙏 Credits

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [image-rs](https://github.com/image-rs/image) - Image processing
- [printpdf](https://github.com/fschutt/printpdf) - PDF creation
- [pdfium-render](https://crates.io/crates/pdfium-render) - PDF parsing
- [Ollama](https://ollama.ai/) - Local LLM inference
- [fast_image_resize](https://github.com/Cykooz/fast_image_resize) - High-quality upscaling

---

**Made with ❤️ for mobile comic readers** 📱📚
