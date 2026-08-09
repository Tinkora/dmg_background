use std::io::{Cursor, Write};

use crate::{
    BACKGROUND_1X_PATH, BACKGROUND_2X_PATH, CoreError, LayoutDocument, layout_schema_json,
};

pub const PREVIEW_PATH: &str = "preview.png";
pub const LAYOUT_PATH: &str = "dmg_layout.json";
pub const README_PATH: &str = "README.txt";

/// Exports the five-member ZIP after validating the layout and decoding every PNG asset.
pub fn export_contract_zip(
    document: &LayoutDocument,
    background_1x: &[u8],
    background_2x: &[u8],
    preview: &[u8],
) -> Result<Vec<u8>, CoreError> {
    document.validate()?;
    let dimensions = document.output_dimensions()?;
    validate_png_asset(BACKGROUND_1X_PATH, background_1x, dimensions.one_x)?;
    validate_png_asset(BACKGROUND_2X_PATH, background_2x, dimensions.two_x)?;
    validate_png_asset(PREVIEW_PATH, preview, dimensions.one_x)?;

    let layout = serde_json::to_vec_pretty(document)
        .map_err(|error| CoreError::LayoutSerialization(error.to_string()))?;
    let mut bytes = Cursor::new(Vec::new());
    let mut archive = zip::ZipWriter::new(&mut bytes);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);

    for (path, content) in [
        (BACKGROUND_1X_PATH, background_1x),
        (BACKGROUND_2X_PATH, background_2x),
        (PREVIEW_PATH, preview),
    ] {
        archive
            .start_file(path, options)
            .map_err(|error| CoreError::ZipExport(error.to_string()))?;
        archive
            .write_all(content)
            .map_err(|error| CoreError::ZipExport(error.to_string()))?;
    }

    archive
        .start_file(LAYOUT_PATH, options)
        .map_err(|error| CoreError::ZipExport(error.to_string()))?;
    archive
        .write_all(&layout)
        .map_err(|error| CoreError::ZipExport(error.to_string()))?;

    archive
        .start_file(README_PATH, options)
        .map_err(|error| CoreError::ZipExport(error.to_string()))?;
    archive
        .write_all(readme_text().as_bytes())
        .map_err(|error| CoreError::ZipExport(error.to_string()))?;

    archive
        .finish()
        .map_err(|error| CoreError::ZipExport(error.to_string()))?;
    Ok(bytes.into_inner())
}

fn validate_png_asset(
    asset: &'static str,
    bytes: &[u8],
    expected: crate::Window,
) -> Result<(), CoreError> {
    let invalid = || CoreError::InvalidPngAsset { asset };
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.ignore_checksums(false);
    decoder.set_ignore_text_chunk(true);
    decoder.set_ignore_iccp_chunk(true);
    let mut reader = decoder.read_info().map_err(|_| invalid())?;
    if reader.info().animation_control.is_some() {
        return Err(invalid());
    }
    let actual_width = reader.info().width;
    let actual_height = reader.info().height;
    if actual_width != expected.width || actual_height != expected.height {
        return Err(CoreError::PngDimensionMismatch {
            asset,
            expected_width: expected.width,
            expected_height: expected.height,
            actual_width,
            actual_height,
        });
    }
    while reader.next_row().map_err(|_| invalid())?.is_some() {}
    reader.finish().map_err(|_| invalid())?;
    Ok(())
}

/// Returns the JSON Schema for the current ZIP contract, for independent Runners or test tooling.
pub const fn layout_schema() -> &'static str {
    layout_schema_json()
}

const fn readme_text() -> &'static str {
    "dmg_background export contract v1\n\nUse dmg_layout.json with a macOS Runner. preview.png is for review only; use .background/background.png as the Finder background.\n"
}
