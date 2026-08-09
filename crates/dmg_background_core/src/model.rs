use serde::{Deserialize, Serialize};

use crate::CoreError;

pub const SCHEMA_VERSION: u32 = 1;
pub const SCHEMA_URI: &str = "https://tinkora.github.io/dmg_background/schema/dmg-layout-v1.json";
pub const COORDINATE_SPACE: &str = "finder_content_logical_points";
pub const BACKGROUND_1X_PATH: &str = ".background/background.png";
pub const BACKGROUND_2X_PATH: &str = ".background/background@2x.png";
pub const MAX_RETINA_PIXELS: u64 = 16_777_216;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutDocument {
    #[serde(rename = "$schema")]
    pub schema_uri: String,
    pub schema_version: u32,
    pub coordinate_space: String,
    pub window: Window,
    pub icon_size: u32,
    pub background: BackgroundAssets,
    pub items: Vec<LayoutItem>,
    pub texts: Vec<TextElement>,
    pub guides: Vec<Guide>,
    pub export: ExportOptions,
}

impl LayoutDocument {
    /// Creates a minimal valid "drag app to Applications" layout.
    pub fn app_to_applications(
        window: Window,
        volume_name: impl Into<String>,
    ) -> Result<Self, CoreError> {
        if !(320..=4096).contains(&window.width) || !(320..=4096).contains(&window.height) {
            return Err(CoreError::InvalidWindowSize {
                width: window.width,
                height: window.height,
            });
        }
        retina_dimensions(window.width, window.height)?;

        let center_y = window.height / 2;
        let left_x = window.width / 8 * 3;
        let right_x = window.width / 8 * 5;
        let volume_name = volume_name.into();

        Ok(Self {
            schema_uri: SCHEMA_URI.to_owned(),
            schema_version: SCHEMA_VERSION,
            coordinate_space: COORDINATE_SPACE.to_owned(),
            window,
            icon_size: 128,
            background: BackgroundAssets::default(),
            items: vec![
                LayoutItem {
                    id: "app".to_owned(),
                    kind: ItemKind::Application,
                    x: left_x,
                    y: center_y,
                    label: format!("{volume_name}.app"),
                },
                LayoutItem {
                    id: "applications".to_owned(),
                    kind: ItemKind::ApplicationsAlias,
                    x: right_x,
                    y: center_y,
                    label: "Applications".to_owned(),
                },
            ],
            texts: Vec::new(),
            guides: Vec::new(),
            export: ExportOptions {
                volume_name,
                hidden_files_policy: HiddenFilesPolicy::BackgroundDirectory,
            },
        })
    }

    pub fn output_dimensions(&self) -> Result<OutputDimensions, CoreError> {
        retina_dimensions(self.window.width, self.window.height)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Window {
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct BackgroundAssets {
    pub one_x: String,
    pub two_x: String,
}

impl Default for BackgroundAssets {
    fn default() -> Self {
        Self {
            one_x: BACKGROUND_1X_PATH.to_owned(),
            two_x: BACKGROUND_2X_PATH.to_owned(),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct LayoutItem {
    pub id: String,
    pub kind: ItemKind,
    pub x: u32,
    pub y: u32,
    pub label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ItemKind {
    Application,
    ApplicationsAlias,
    ExtraFile,
}

impl ItemKind {
    pub const fn contract_name(self) -> &'static str {
        match self {
            Self::Application => "application",
            Self::ApplicationsAlias => "applications_alias",
            Self::ExtraFile => "extra_file",
        }
    }
}

/// A text rectangle renderable on the formal background; `x/y` are top-left logical-point coordinates.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextElement {
    pub id: String,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub font_size: u32,
    pub content: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Guide {
    pub axis: GuideAxis,
    pub position: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GuideAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportOptions {
    pub volume_name: String,
    pub hidden_files_policy: HiddenFilesPolicy,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HiddenFilesPolicy {
    BackgroundDirectory,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OutputDimensions {
    pub one_x: Window,
    pub two_x: Window,
}

/// Computes 1x/2x output from logical window dimensions, explicitly guarding against integer overflow.
pub fn retina_dimensions(width: u32, height: u32) -> Result<OutputDimensions, CoreError> {
    let two_x_width = width
        .checked_mul(2)
        .ok_or(CoreError::OutputDimensionOverflow)?;
    let two_x_height = height
        .checked_mul(2)
        .ok_or(CoreError::OutputDimensionOverflow)?;
    let pixels = u64::from(two_x_width) * u64::from(two_x_height);
    if pixels > MAX_RETINA_PIXELS {
        return Err(CoreError::OutputPixelBudgetExceeded {
            pixels,
            max_pixels: MAX_RETINA_PIXELS,
        });
    }

    Ok(OutputDimensions {
        one_x: Window { width, height },
        two_x: Window {
            width: two_x_width,
            height: two_x_height,
        },
    })
}
