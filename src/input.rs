use anyhow::{Context, Result};
use image::GenericImageView;
use pdfium_render::prelude::*;
use std::path::Path;

use crate::types::Page;

/// Load comic from PDF or directory of images
pub fn load_comic(input_path: &Path) -> Result<Vec<Page>> {
    if input_path.is_file() {
        if let Some(ext) = input_path.extension() {
            if ext == "pdf" {
                return load_from_pdf(input_path);
            } else {
                // Single image file
                let img = image::open(input_path)
                    .context("Failed to load image file")?;
                return Ok(vec![Page { number: 1, image: img }]);
            }
        }
    } else if input_path.is_dir() {
        return load_from_directory(input_path);
    }
    
    anyhow::bail!("Input must be a PDF file, image file, or directory of images");
}

/// Load pages from a PDF file
fn load_from_pdf(pdf_path: &Path) -> Result<Vec<Page>> {
    let pdfium = Pdfium::new(
        Pdfium::bind_to_library(Pdfium::pdfium_platform_library_name_at_path("./"))
            .or_else(|_| Pdfium::bind_to_system_library())
            .context("Failed to bind to PDFium library")?
    );
    
    let document = pdfium.load_pdf_from_file(pdf_path, None)
        .context("Failed to load PDF document")?;
    
    let mut pages = Vec::new();
    
    for page_index in 0..document.pages().len() {
        let page = document.pages().get(page_index)
            .context(format!("Failed to get page {}", page_index))?;
        
        // Render page to image at high DPI for quality
        let render_config = PdfRenderConfig::new()
            .set_target_width(2000)
            .set_maximum_height(3000)
            .rotate_if_landscape(PdfPageRenderRotation::None, true);
        
        let bitmap = page.render_with_config(&render_config)
            .context("Failed to render page")?;
        
        // Convert bitmap to DynamicImage
        let image = bitmap.as_image();
        
        pages.push(Page {
            number: (page_index + 1) as usize,
            image,
        });
    }
    
    log::info!("Loaded {} pages from PDF", pages.len());
    Ok(pages)
}

/// Load images from a directory
fn load_from_directory(dir_path: &Path) -> Result<Vec<Page>> {
    let mut entries: Vec<_> = std::fs::read_dir(dir_path)
        .context("Failed to read directory")?
        .filter_map(|e| e.ok())
        .filter(|e| {
            if let Some(ext) = e.path().extension() {
                matches!(
                    ext.to_string_lossy().to_lowercase().as_str(),
                    "jpg" | "jpeg" | "png" | "webp"
                )
            } else {
                false
            }
        })
        .collect();
    
    // Sort by filename
    entries.sort_by_key(|e| e.path());
    
    let mut pages = Vec::new();
    
    for (idx, entry) in entries.iter().enumerate() {
        let img = image::open(entry.path())
            .context(format!("Failed to load image: {:?}", entry.path()))?;
        
        pages.push(Page {
            number: idx + 1,
            image: img,
        });
    }
    
    log::info!("Loaded {} images from directory", pages.len());
    Ok(pages)
}
