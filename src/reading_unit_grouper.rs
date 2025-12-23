use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::types::{Panel, ReadingUnit, LayoutStrategy, PanelMetadata, ReadingUnitGroup};

/// Group panels into reading units using Ollama LLM
pub async fn group_panels(
    panels: Vec<Panel>,
    ollama_url: &str,
    model_name: &str,
    verbose: bool,
) -> Result<Vec<ReadingUnit>> {
    if panels.is_empty() {
        return Ok(Vec::new());
    }
    
    // Create metadata for AI analysis
    let panel_metadata: Vec<PanelMetadata> = panels
        .iter()
        .enumerate()
        .map(|(idx, panel)| PanelMetadata {
            panel_id: idx,
            page_number: panel.page_number,
            position: panel.bbox,
            relative_position: determine_relative_position(&panel.bbox),
        })
        .collect();
    
    // Build prompt for Ollama
    let prompt = build_grouping_prompt(&panel_metadata);
    
    if verbose {
        log::info!("  Sending request to Ollama...");
    }
    
    // Call Ollama API
    let groups = query_ollama(ollama_url, model_name, &prompt, verbose).await?;
    
    // Convert AI response to reading units
    let reading_units = create_reading_units(panels, groups)?;
    
    if verbose {
        log::info!("  Created {} reading units", reading_units.len());
    }
    
    Ok(reading_units)
}

/// Determine relative position of a panel (for AI context)
fn determine_relative_position(bbox: &crate::types::Rectangle) -> String {
    // Simple heuristic based on position
    let vertical = if bbox.y < 500 {
        "top"
    } else if bbox.y < 1500 {
        "middle"
    } else {
        "bottom"
    };
    
    let horizontal = if bbox.x < 500 {
        "left"
    } else if bbox.x < 1500 {
        "center"
    } else {
        "right"
    };
    
    format!("{}-{}", vertical, horizontal)
}

/// Build prompt for Ollama to group panels
fn build_grouping_prompt(metadata: &[PanelMetadata]) -> String {
    let mut prompt = String::from(
        "You are analyzing a comic or manga. Your task is to group panels into 'reading units' \
        that should be displayed together on a mobile screen.\n\n\
        Rules for grouping:\n\
        1. Panels on the same page that are visually close should often be grouped\n\
        2. A reading unit should contain 1-3 panels maximum (to fit on mobile screen)\n\
        3. Keep panels from different pages separate unless they're clearly a continuous action\n\
        4. Single large panels should be their own unit\n\n\
        Panel data:\n"
    );
    
    for panel in metadata {
        prompt.push_str(&format!(
            "- Panel {}: page {}, position ({}, {}), size {}x{}, location: {}\n",
            panel.panel_id,
            panel.page_number,
            panel.position.x,
            panel.position.y,
            panel.position.width,
            panel.position.height,
            panel.relative_position
        ));
    }
    
    prompt.push_str(
        "\nRespond with ONLY a JSON array of reading units in this exact format:\n\
        [{\"unit_id\": 0, \"panel_ids\": [0], \"layout\": \"SinglePanel\"},\n\
         {\"unit_id\": 1, \"panel_ids\": [1, 2], \"layout\": \"VerticalStack\"}]\n\n\
        Available layouts: \"SinglePanel\", \"VerticalStack\", \"Grid2x1\"\n\n\
        JSON output:"
    );
    
    prompt
}

/// Ollama API request/response types
#[derive(Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
    format: String,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
}

/// Query Ollama API
async fn query_ollama(
    base_url: &str,
    model: &str,
    prompt: &str,
    verbose: bool,
) -> Result<Vec<ReadingUnitGroup>> {
    let client = Client::new();
    let url = format!("{}/api/generate", base_url);
    
    let request = OllamaRequest {
        model: model.to_string(),
        prompt: prompt.to_string(),
        stream: false,
        format: "json".to_string(),
    };
    
    if verbose {
        log::debug!("Ollama request: {:?}", request.prompt);
    }
    
    let response = client
        .post(&url)
        .json(&request)
        .send()
        .await
        .context("Failed to send request to Ollama")?;
    
    if !response.status().is_success() {
        anyhow::bail!("Ollama request failed with status: {}", response.status());
    }
    
    let ollama_response: OllamaResponse = response
        .json()
        .await
        .context("Failed to parse Ollama response")?;
    
    if verbose {
        log::debug!("Ollama response: {}", ollama_response.response);
    }
    
    // Parse JSON response
    let groups: Vec<ReadingUnitGroup> = serde_json::from_str(&ollama_response.response)
        .context("Failed to parse reading unit groups from Ollama response")?;
    
    Ok(groups)
}

/// Convert AI grouping response to reading units
fn create_reading_units(
    panels: Vec<Panel>,
    groups: Vec<ReadingUnitGroup>,
) -> Result<Vec<ReadingUnit>> {
    let mut reading_units = Vec::new();
    
    for group in groups {
        let unit_panels: Vec<Panel> = group
            .panel_ids
            .iter()
            .filter_map(|&id| {
                if id < panels.len() {
                    Some(panels[id].clone())
                } else {
                    None
                }
            })
            .collect();
        
        if unit_panels.is_empty() {
            continue;
        }
        
        reading_units.push(ReadingUnit {
            id: group.unit_id,
            panels: unit_panels,
            layout_strategy: group.layout,
        });
    }
    
    // Fallback: if AI didn't group all panels, create individual units
    if reading_units.is_empty() {
        log::warn!("AI grouping failed, falling back to individual panels");
        for (idx, panel) in panels.into_iter().enumerate() {
            reading_units.push(ReadingUnit {
                id: idx,
                panels: vec![panel],
                layout_strategy: LayoutStrategy::SinglePanel,
            });
        }
    }
    
    Ok(reading_units)
}
