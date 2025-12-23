use anyhow::{Context, Result};
use image::{DynamicImage, GenericImageView, imageops, Rgba, RgbaImage};
use fast_image_resize as fr;

use crate::types::{ReadingUnit, ProcessedUnit, LayoutStrategy};

/// Process all reading units
pub fn process_reading_units(
    units: Vec<ReadingUnit>,
    aspect_ratio_str: String,
    upscale_factor: u32,
    verbose: bool,
) -> Result<Vec<ProcessedUnit>> {
    let target_aspect = parse_aspect_ratio(&aspect_ratio_str)?;
    
    let mut processed = Vec::new();
    
    for unit in units {
        if verbose {
            log::info!("  Processing reading unit {} with {} panels", unit.id, unit.panels.len());
        }
        
        let processed_image = process_reading_unit(&unit, target_aspect, upscale_factor, verbose)?;
        
        processed.push(ProcessedUnit {
            id: unit.id,
            image: processed_image,
        });
    }
    
    Ok(processed)
}

/// Parse aspect ratio string like "9:16" to float
fn parse_aspect_ratio(ratio_str: &str) -> Result<f32> {
    let parts: Vec<&str> = ratio_str.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid aspect ratio format. Expected 'width:height' (e.g., '9:16')");
    }
    
    let width: f32 = parts[0].parse().context("Invalid width in aspect ratio")?;
    let height: f32 = parts[1].parse().context("Invalid height in aspect ratio")?;
    
    Ok(width / height)
}

/// Process a single reading unit
fn process_reading_unit(
    unit: &ReadingUnit,
    target_aspect: f32,
    upscale_factor: u32,
    verbose: bool,
) -> Result<DynamicImage> {
    // Step 1: Merge panels according to layout strategy
    let merged = merge_panels(&unit.panels, &unit.layout_strategy)?;
    
    // Step 2: Smart crop to target aspect ratio
    let cropped = smart_crop_to_aspect(&merged, target_aspect)?;
    
    // Step 3: Upscale
    let upscaled = upscale_image(&cropped, upscale_factor)?;
    
    // Step 4: Enhance lighting
    let enhanced = enhance_lighting(&upscaled);
    
    if verbose {
        log::info!("    Processed: {}x{} -> {}x{}",
            merged.width(), merged.height(),
            enhanced.width(), enhanced.height()
        );
    }
    
    Ok(enhanced)
}

/// Merge multiple panels according to layout strategy
fn merge_panels(panels: &[crate::types::Panel], layout: &LayoutStrategy) -> Result<DynamicImage> {
    if panels.is_empty() {
        anyhow::bail!("Cannot merge empty panels");
    }
    
    if panels.len() == 1 {
        return Ok(panels[0].image_data.clone());
    }
    
    match layout {
        LayoutStrategy::SinglePanel => Ok(panels[0].image_data.clone()),
        LayoutStrategy::VerticalStack => merge_vertical(panels),
        LayoutStrategy::Grid2x1 => merge_grid_2x1(panels),
        LayoutStrategy::Custom => merge_vertical(panels), // Default to vertical
    }
}

/// Merge panels vertically
fn merge_vertical(panels: &[crate::types::Panel]) -> Result<DynamicImage> {
    // Calculate total height and max width
    let max_width = panels.iter().map(|p| p.image_data.width()).max().unwrap_or(0);
    let total_height: u32 = panels.iter().map(|p| p.image_data.height()).sum();
    
    if max_width == 0 || total_height == 0 {
        anyhow::bail!("Invalid panel dimensions");
    }
    
    // Create new image
    let mut merged = RgbaImage::new(max_width, total_height);
    
    // Stack panels vertically
    let mut y_offset = 0;
    for panel in panels {
        let panel_rgba = panel.image_data.to_rgba8();
        
        // Center panel horizontally if narrower than max_width
        let x_offset = (max_width - panel.image_data.width()) / 2;
        
        imageops::overlay(&mut merged, &panel_rgba, x_offset as i64, y_offset as i64);
        y_offset += panel.image_data.height();
    }
    
    Ok(DynamicImage::ImageRgba8(merged))
}

/// Merge panels in 2x1 grid (two panels side by side)
fn merge_grid_2x1(panels: &[crate::types::Panel]) -> Result<DynamicImage> {
    if panels.len() < 2 {
        return merge_vertical(panels);
    }
    
    let panel1 = &panels[0].image_data;
    let panel2 = &panels[1].image_data;
    
    // Calculate dimensions
    let total_width = panel1.width() + panel2.width();
    let max_height = panel1.height().max(panel2.height());
    
    let mut merged = RgbaImage::new(total_width, max_height);
    
    // Place panels side by side
    imageops::overlay(&mut merged, &panel1.to_rgba8(), 0, 0);
    imageops::overlay(&mut merged, &panel2.to_rgba8(), panel1.width() as i64, 0);
    
    // If there are more panels, stack them vertically below
    if panels.len() > 2 {
        let remaining: Vec<_> = panels[2..].to_vec();
        let bottom_merged = merge_vertical(&remaining)?;
        
        // Create new image with both parts
        let final_height = max_height + bottom_merged.height();
        let final_width = total_width.max(bottom_merged.width());
        
        let mut final_image = RgbaImage::new(final_width, final_height);
        imageops::overlay(&mut final_image, &merged, 0, 0);
        imageops::overlay(&mut final_image, &bottom_merged.to_rgba8(), 0, max_height as i64);
        
        return Ok(DynamicImage::ImageRgba8(final_image));
    }
    
    Ok(DynamicImage::ImageRgba8(merged))
}

/// Smart crop image to target aspect ratio
fn smart_crop_to_aspect(image: &DynamicImage, target_aspect: f32) -> Result<DynamicImage> {
    let current_aspect = image.width() as f32 / image.height() as f32;
    
    // If already close to target aspect, no cropping needed
    if (current_aspect - target_aspect).abs() < 0.01 {
        return Ok(image.clone());
    }
    
    let (crop_width, crop_height) = if current_aspect > target_aspect {
        // Image is wider than target, crop width
        let new_width = (image.height() as f32 * target_aspect) as u32;
        (new_width, image.height())
    } else {
        // Image is taller than target, crop height
        let new_height = (image.width() as f32 / target_aspect) as u32;
        (image.width(), new_height)
    };
    
    // Center crop
    let x_offset = (image.width().saturating_sub(crop_width)) / 2;
    let y_offset = (image.height().saturating_sub(crop_height)) / 2;
    
    Ok(image.crop_imm(x_offset, y_offset, crop_width, crop_height))
}

/// Upscale image using fast_image_resize
fn upscale_image(image: &DynamicImage, factor: u32) -> Result<DynamicImage> {
    if factor <= 1 {
        return Ok(image.clone());
    }
    
    let src_width = image.width();
    let src_height = image.height();
    let dst_width = src_width * factor;
    let dst_height = src_height * factor;
    
    // Convert to RGBA for processing
    let src_image = image.to_rgba8();
    
    // Create source and destination images for fast_image_resize
    let src_fr = fr::Image::from_vec_u8(
        src_width,
        src_height,
        src_image.into_raw(),
        fr::PixelType::U8x4,
    )?;
    
    let mut dst_fr = fr::Image::new(dst_width, dst_height, fr::PixelType::U8x4);
    
    // Create resizer and resize using Lanczos3 for high quality
    let mut resizer = fr::Resizer::new();
    resizer.resize(
        &src_fr.view(),
        &mut dst_fr.view_mut(),
        Some(&fr::ResizeOptions::new().resize_alg(fr::ResizeAlg::Convolution(fr::FilterType::Lanczos3)))
    )?;
    
    // Convert back to DynamicImage
    let buffer = dst_fr.into_vec();
    let rgba_image = RgbaImage::from_raw(dst_width, dst_height, buffer)
        .context("Failed to create image from upscaled buffer")?;
    
    Ok(DynamicImage::ImageRgba8(rgba_image))
}

/// Enhance lighting (brightness, contrast, sharpness)
fn enhance_lighting(image: &DynamicImage) -> DynamicImage {
    // Apply brightness and contrast adjustments
    let brightened = imageops::brighten(image, 5); // Slight brightness boost
    
    // Note: For more advanced enhancements (sharpening, etc.),
    // we could integrate additional image processing libraries
    // For MVP, basic brightness adjustment is sufficient
    
    DynamicImage::ImageRgba8(brightened)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parse_aspect_ratio() {
        assert!((parse_aspect_ratio("9:16").unwrap() - 0.5625).abs() < 0.01);
        assert!((parse_aspect_ratio("16:9").unwrap() - 1.7777).abs() < 0.01);
    }
}
