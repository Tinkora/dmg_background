use thiserror::Error;

/// Stable error type for the layout contract.
#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum CoreError {
    #[error("Unsupported schema URI: {0}")]
    UnsupportedSchemaUri(String),

    #[error("Unsupported schema version: {0}")]
    UnsupportedSchemaVersion(u32),

    #[error("Unsupported coordinate space: {0}")]
    UnsupportedCoordinateSpace(String),

    #[error("Window size {width}x{height} outside the 320..=4096 logical-point range")]
    InvalidWindowSize { width: u32, height: u32 },

    #[error("Icon size must be an even number in the 2..=4096 range")]
    InvalidIconSize,

    #[error("Missing required item: {0}")]
    MissingRequiredItem(&'static str),

    #[error("Duplicate required item: {0}")]
    DuplicateRequiredItem(&'static str),

    #[error("Item ID must not be empty")]
    EmptyItemId,

    #[error("Duplicate item ID: {0}")]
    DuplicateItemId(String),

    #[error("Icon rect for item {id} is outside the window bounds")]
    ItemOutOfBounds { id: String },

    #[error("Text ID must not be empty")]
    EmptyTextId,

    #[error("Duplicate text ID: {0}")]
    DuplicateTextId(String),

    #[error("Invalid rectangle or font size for text {id}")]
    InvalidTextBounds { id: String },

    #[error("Guide position is outside the window bounds")]
    GuideOutOfBounds,

    #[error("Background paths do not conform to Schema 1 fixed contract")]
    InvalidBackgroundPaths,

    #[error("Output dimension calculation overflow")]
    OutputDimensionOverflow,

    #[error("Retina output requires {pixels} pixels, above the {max_pixels} pixel budget")]
    OutputPixelBudgetExceeded { pixels: u64, max_pixels: u64 },

    #[error("{asset} is not a decodable, non-animated PNG")]
    InvalidPngAsset { asset: &'static str },

    #[error(
        "{asset} is {actual_width}x{actual_height}, expected {expected_width}x{expected_height}"
    )]
    PngDimensionMismatch {
        asset: &'static str,
        expected_width: u32,
        expected_height: u32,
        actual_width: u32,
        actual_height: u32,
    },

    #[error("Layout JSON serialization failed: {0}")]
    LayoutSerialization(String),

    #[error("ZIP export failed: {0}")]
    ZipExport(String),
}

impl CoreError {
    /// Returns the stable machine error code exposed by the WebAssembly boundary.
    pub const fn code(&self) -> &'static str {
        match self {
            Self::UnsupportedSchemaUri(_) => "UNSUPPORTED_SCHEMA_URI",
            Self::UnsupportedSchemaVersion(_) => "UNSUPPORTED_SCHEMA_VERSION",
            Self::UnsupportedCoordinateSpace(_) => "UNSUPPORTED_COORDINATE_SPACE",
            Self::InvalidWindowSize { .. } => "INVALID_WINDOW_SIZE",
            Self::InvalidIconSize => "INVALID_ICON_SIZE",
            Self::MissingRequiredItem(_) => "MISSING_REQUIRED_ITEM",
            Self::DuplicateRequiredItem(_) => "DUPLICATE_REQUIRED_ITEM",
            Self::EmptyItemId => "EMPTY_ITEM_ID",
            Self::DuplicateItemId(_) => "DUPLICATE_ITEM_ID",
            Self::ItemOutOfBounds { .. } => "ITEM_OUT_OF_BOUNDS",
            Self::EmptyTextId => "EMPTY_TEXT_ID",
            Self::DuplicateTextId(_) => "DUPLICATE_TEXT_ID",
            Self::InvalidTextBounds { .. } => "INVALID_TEXT_BOUNDS",
            Self::GuideOutOfBounds => "GUIDE_OUT_OF_BOUNDS",
            Self::InvalidBackgroundPaths => "INVALID_BACKGROUND_PATHS",
            Self::OutputDimensionOverflow => "OUTPUT_DIMENSION_OVERFLOW",
            Self::OutputPixelBudgetExceeded { .. } => "OUTPUT_PIXEL_BUDGET_EXCEEDED",
            Self::InvalidPngAsset { .. } => "INVALID_PNG_ASSET",
            Self::PngDimensionMismatch { .. } => "PNG_DIMENSION_MISMATCH",
            Self::LayoutSerialization(_) => "LAYOUT_SERIALIZATION",
            Self::ZipExport(_) => "ZIP_EXPORT",
        }
    }
}
