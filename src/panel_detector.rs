use anyhow::{Context, Result};
use image::{DynamicImage, GrayImage};
use imageproc::contours::find_contours;
use rayon::prelude::*;

use crate::types::{Page, Panel, Rectangle};

/// Detect panels in all pages using computer vision (parallel processing)
pub fn detect_panels(pages: &[Page], verbose: bool) -> Result<Vec<Panel>> {
    // Process pages in parallel using rayon
    let all_panels: Vec<Panel> = pages
        .par_iter()
        .enumerate()
        .flat_map(|(_idx, page)| {
            if verbose {
                log::info!("  Processing page {}", page.number);
            }
            
            match detect_panels_in_page(page, verbose) {
                Ok(panels) => panels,
                Err(e) => {
                    log::warn!("Failed to detect panels on page {}: {}", page.number, e);
                    vec![]
                }
            }
        })
        .collect();
    
    Ok(all_panels)
}

/// Detect panels in a single page
fn detect_panels_in_page(page: &Page, verbose: bool) -> Result<Vec<Panel>> {
    // Convert to grayscale
    let gray_image = page.image.to_luma8();
    
    // Apply threshold to detect white gutters (spaces between panels)
    // Comics typically have white backgrounds/gutters
    let threshold_value = 200; // High value to detect white areas
    let binary =  imageproc::contrast::threshold(
        &gray_image,
        threshold_value,
        imageproc::contrast::ThresholdType::Binary
    );
    
    // Invert: we want to find dark (panel content) regions
    let inverted = invert_binary(&binary);
    
    // Find contours
    let contours = find_contours::<u32>(&inverted);
    
    if verbose {
        log::info!("    Found {} contours", contours.len());
    }
    
    // Convert contours to bounding boxes
    let mut panels = Vec::new();
    
    for (_idx, contour) in contours.iter().enumerate() {
        if contour.points.is_empty() {
            continue;
        }
        
        let bbox = contour_to_bbox(&contour.points);
        
        // Filter out very small panels (likely noise)
        let min_panel_size = 50 * 50; // Minimum 50x50 pixels
        if bbox.area() < min_panel_size {
            continue;
        }
        
        // Filter out panels that are almost the full page (likely background)
        let page_area = page.image.width() * page.image.height();
        if bbox.area() > (page_area as f32 * 0.95) as u32 {
            continue;
        }
        
        // Extract panel image
        let panel_image = extract_panel_image(&page.image, &bbox)?;
        
        panels.push(Panel {
            bbox,
            page_number: page.number,
            confidence: 0.9, // CV-based detection has fixed confidence
            image_data: panel_image,
        });
    }
    
    // If no panels detected, treat entire page as one panel
    if panels.is_empty() {
        if verbose {
            log::info!("    No panels detected, using full page as single panel");
        }
        
        let bbox = Rectangle::new(0, 0, page.image.width(), page.image.height());
        panels.push(Panel {
            bbox,
            page_number: page.number,
            confidence: 1.0,
            image_data: page.image.clone(),
        });
    }
    
    // Sort panels by reading order (top to bottom, left to right)
    panels.sort_by(|a, b| {
        let y_diff = a.bbox.y as i32 - b.bbox.y as i32;
        if y_diff.abs() < 50 { // If on similar y-level, sort by x
            a.bbox.x.cmp(&b.bbox.x)
        } else {
            a.bbox.y.cmp(&b.bbox.y)
        }
    });
    
    if verbose {
        log::info!("    Detected {} valid panels", panels.len());
    }
    
    Ok(panels)
}

/// Invert a binary image
fn invert_binary(image: &GrayImage) -> GrayImage {
    let mut inverted = image.clone();
    for pixel in inverted.pixels_mut() {
        pixel.0[0] = 255 - pixel.0[0];
    }
    inverted
}

/// Convert contour points to bounding box
fn contour_to_bbox(points: &[imageproc::point::Point<u32>]) -> Rectangle {
    let min_x = points.iter().map(|p| p.x).min().unwrap_or(0);
    let max_x = points.iter().map(|p| p.x).max().unwrap_or(0);
    let min_y = points.iter().map(|p| p.y).min().unwrap_or(0);
    let max_y = points.iter().map(|p| p.y).max().unwrap_or(0);
    
    Rectangle::new(
        min_x,
        min_y,
        max_x.saturating_sub(min_x),
        max_y.saturating_sub(min_y),
    )
}

/// Extract panel image from page using bounding box
fn extract_panel_image(page_image: &DynamicImage, bbox: &Rectangle) -> Result<DynamicImage> {
    // Ensure bbox is within image bounds
    let img_width = page_image.width();
    let img_height = page_image.height();
    
    let x = bbox.x.min(img_width.saturating_sub(1));
    let y = bbox.y.min(img_height.saturating_sub(1));
    let width = bbox.width.min(img_width - x);
    let height = bbox.height.min(img_height - y);
    
    let cropped = page_image.crop_imm(x, y, width, height);
    
    Ok(cropped)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_rectangle_area() {
        let rect = Rectangle::new(0, 0, 100, 200);
        assert_eq!(rect.area(), 20000);
    }
    
    #[test]
    fn test_rectangle_aspect_ratio() {
        let rect = Rectangle::new(0, 0, 16, 9);
        assert!((rect.aspect_ratio() - 16.0/9.0).abs() < 0.01);
    }
}
