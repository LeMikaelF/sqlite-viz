use std::path::Path;
use crate::error::Result;
use crate::visualization::data::VizData;
use crate::visualization::templates::{HTML_TEMPLATE, STYLES_CSS, VIZ_JS};

/// Generate the HTML visualization file
pub fn generate_html(viz_data: &VizData, output_path: &Path) -> Result<()> {
    let json_data = serde_json::to_string(viz_data)?;

    // Inline CSS and JS into HTML
    let html = HTML_TEMPLATE
        .replace("/* __STYLES_PLACEHOLDER__ */", STYLES_CSS)
        .replace("/* __VIZ_JS_PLACEHOLDER__ */", VIZ_JS)
        .replace("\"__DATA_PLACEHOLDER__\"", &json_data);

    std::fs::write(output_path, html)?;
    Ok(())
}
