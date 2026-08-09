mod error;
mod export;
mod model;
mod validation;

pub use error::CoreError;
pub use export::{LAYOUT_PATH, PREVIEW_PATH, README_PATH, export_contract_zip, layout_schema};
pub use model::{
    BACKGROUND_1X_PATH, BACKGROUND_2X_PATH, BackgroundAssets, COORDINATE_SPACE, ExportOptions,
    Guide, GuideAxis, HiddenFilesPolicy, ItemKind, LayoutDocument, LayoutItem, MAX_RETINA_PIXELS,
    OutputDimensions, SCHEMA_URI, SCHEMA_VERSION, TextElement, Window, retina_dimensions,
};

/// Returns JSON Schema 1 as shipped with the crate.
pub const fn layout_schema_json() -> &'static str {
    include_str!("../schema/dmg_layout.schema.json")
}
