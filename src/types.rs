use image::DynamicImage;
use serde::{Deserialize, Serialize};

/// Represents a rectangular bounding box
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl Rectangle {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn area(&self) -> u32 {
        self.width * self.height
    }

    pub fn aspect_ratio(&self) -> f32 {
        self.width as f32 / self.height as f32
    }
}

/// A single comic/manga page
#[derive(Debug)]
pub struct Page {
    pub number: usize,
    pub image: DynamicImage,
}

/// A detected panel within a page
#[derive(Debug, Clone)]
pub struct Panel {
    pub bbox: Rectangle,
    pub page_number: usize,
    pub confidence: f32,
    pub image_data: DynamicImage,
}

/// Layout strategy for arranging panels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LayoutStrategy {
    SinglePanel,
    VerticalStack,
    Grid2x1,
    Custom,
}

/// A reading unit containing one or more panels
#[derive(Debug)]
pub struct ReadingUnit {
    pub id: usize,
    pub panels: Vec<Panel>,
    pub layout_strategy: LayoutStrategy,
}

/// A processed reading unit ready for PDF
#[derive(Debug)]
pub struct ProcessedUnit {
    pub id: usize,
    pub image: DynamicImage,
}

/// Panel metadata for AI reasoning
#[derive(Debug, Serialize, Deserialize)]
pub struct PanelMetadata {
    pub panel_id: usize,
    pub page_number: usize,
    pub position: Rectangle,
    pub relative_position: String, // "top-left", "center", etc.
}

/// Response from Ollama for reading unit grouping
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadingUnitGroup {
    pub unit_id: usize,
    pub panel_ids: Vec<usize>,
    pub layout: LayoutStrategy,
}
