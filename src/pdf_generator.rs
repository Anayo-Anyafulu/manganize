use anyhow::{Context, Result};
use printpdf::*;
use std::path::Path;
use std::fs;
use image::ImageFormat;

use crate::types::ProcessedUnit;

/// Create PDF from processed reading units
pub fn create_pdf(units: &[ProcessedUnit], output_path: &Path) -> Result<()> {
    if units.is_empty() {
        anyhow::bail!("No reading units to convert to PDF");
    }
    
    // Get dimensions from first unit
    let first_unit = &units[0];
    let page_width = first_unit.image.width() as f32;
    let page_height = first_unit.image.height() as f32;
    
    // Convert pixels to mm at 72 DPI (standard PDF DPI)
    let dpi = 72.0;
    let mm_per_pixel = 25.4 / dpi;
    let page_width_mm = page_width * mm_per_pixel;
    let page_height_mm = page_height * mm_per_pixel;
    
    // Create PDF document
    let (doc, page1, layer1) = PdfDocument::new(
        "Manganized Comic",
        Mm(page_width_mm),
        Mm(page_height_mm),
        "Layer 1",
    );
    
    // Create temp directory for images
    let temp_dir = std::env::temp_dir().join("manganize");
    fs::create_dir_all(&temp_dir)
        .context("Failed to create temp directory")?;
    
    // Add first page
    add_unit_to_page(&doc, page1, layer1, &units[0], page_width_mm, page_height_mm, &temp_dir, 0)?;
    
    // Add remaining pages
    for (idx, unit) in units.iter().enumerate().skip(1) {
        let (page, layer) = doc.add_page(
            Mm(page_width_mm),
            Mm(page_height_mm),
            "Layer 1",
        );
        
        add_unit_to_page(&doc, page, layer, unit, page_width_mm, page_height_mm, &temp_dir, idx)?;
    }
    
    // Save to file
    doc.save(&mut std::io::BufWriter::new(
        std::fs::File::create(output_path)
            .context("Failed to create output PDF file")?
    ))
    .context("Failed to save PDF")?;
    
    // Clean up temp files
    let _ = fs::remove_dir_all(&temp_dir);
    
    Ok(())
}

/// Add a reading unit image to a PDF page
fn add_unit_to_page(
    doc: &PdfDocumentReference,
    page: PdfPageIndex,
    layer: PdfLayerIndex,
    unit: &ProcessedUnit,
    page_width_mm: f32,
    page_height_mm: f32,
    temp_dir: &Path,
    idx: usize,
) -> Result<()> {
    // Save image temporarily as JPEG for embedding
    let temp_file = temp_dir.join(format!("page_{}.jpg", idx));
    unit.image.save_with_format(&temp_file, ImageFormat::Jpeg)
        .context("Failed to save temporary image")?;
    
    // Load image from file using printpdf
    let mut image_file = fs::File::open(&temp_file)
        .context("Failed to open temporary image")?;
    
    // Convert to RGB8 for printpdf
    let rgb_image = unit.image.to_rgb8();
    let width_px = rgb_image.width() as usize;
    let height_px = rgb_image.height() as usize;
    
    // Create ImageXObject manually
    let image_xobject = ImageXObject {
        width: Px(width_px),
        height: Px(height_px),
        color_space: ColorSpace::Rgb,
        bits_per_component: ColorBits::Bit8,
        interpolate: true,
        image_data: rgb_image.into_raw(),
        image_filter: None,
        clipping_bbox: None,
        smask: None,
    };
    
    let image = Image::from(image_xobject);
    
    // Get current layer
    let current_layer = doc.get_page(page).get_layer(layer);
    
    // Calculate scaling to fill page
    let scale_x = page_width_mm / (width_px as f32);
    let scale_y = page_height_mm / (height_px as f32);
    
    // Add image to layer
    image.add_to_layer(
        current_layer,
        ImageTransform {
            translate_x: Some(Mm(0.0)),
            translate_y: Some(Mm(0.0)),
            rotate: None,
            scale_x: Some(scale_x),
            scale_y: Some(scale_y),
            dpi: Some(72.0),
        },
    );
    
    Ok(())
}
