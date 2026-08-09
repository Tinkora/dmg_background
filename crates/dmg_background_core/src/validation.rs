use std::collections::HashSet;

use crate::{
    BACKGROUND_1X_PATH, BACKGROUND_2X_PATH, COORDINATE_SPACE, CoreError, GuideAxis, ItemKind,
    LayoutDocument, SCHEMA_URI, SCHEMA_VERSION,
};

const MIN_WINDOW_SIZE: u32 = 320;
const MAX_WINDOW_SIZE: u32 = 4096;

impl LayoutDocument {
    /// Validates Schema 1 window, required items, unique IDs, background paths, and icon bounds.
    pub fn validate(&self) -> Result<(), CoreError> {
        if self.schema_uri != SCHEMA_URI {
            return Err(CoreError::UnsupportedSchemaUri(self.schema_uri.clone()));
        }
        if self.schema_version != SCHEMA_VERSION {
            return Err(CoreError::UnsupportedSchemaVersion(self.schema_version));
        }
        if self.coordinate_space != COORDINATE_SPACE {
            return Err(CoreError::UnsupportedCoordinateSpace(
                self.coordinate_space.clone(),
            ));
        }
        if !(MIN_WINDOW_SIZE..=MAX_WINDOW_SIZE).contains(&self.window.width)
            || !(MIN_WINDOW_SIZE..=MAX_WINDOW_SIZE).contains(&self.window.height)
        {
            return Err(CoreError::InvalidWindowSize {
                width: self.window.width,
                height: self.window.height,
            });
        }
        if self.icon_size < 2 || self.icon_size > MAX_WINDOW_SIZE || self.icon_size % 2 != 0 {
            return Err(CoreError::InvalidIconSize);
        }
        if self.background.one_x != BACKGROUND_1X_PATH
            || self.background.two_x != BACKGROUND_2X_PATH
        {
            return Err(CoreError::InvalidBackgroundPaths);
        }

        let mut ids = HashSet::with_capacity(self.items.len());
        let mut applications = 0_u32;
        let mut aliases = 0_u32;
        for item in &self.items {
            if item.id.is_empty() {
                return Err(CoreError::EmptyItemId);
            }
            if !ids.insert(item.id.as_str()) {
                return Err(CoreError::DuplicateItemId(item.id.clone()));
            }
            match item.kind {
                ItemKind::Application => applications += 1,
                ItemKind::ApplicationsAlias => aliases += 1,
                ItemKind::ExtraFile => {}
            }
            if !item_fits(self, item.x, item.y) {
                return Err(CoreError::ItemOutOfBounds {
                    id: item.id.clone(),
                });
            }
        }

        require_exactly_one(applications, "application")?;
        require_exactly_one(aliases, "applications_alias")?;

        let mut text_ids = HashSet::with_capacity(self.texts.len());
        for text in &self.texts {
            if text.id.is_empty() {
                return Err(CoreError::EmptyTextId);
            }
            if !text_ids.insert(text.id.as_str()) {
                return Err(CoreError::DuplicateTextId(text.id.clone()));
            }
            let right = text.x.checked_add(text.width);
            let bottom = text.y.checked_add(text.height);
            if text.width == 0
                || text.height == 0
                || text.font_size == 0
                || right.is_none_or(|value| value > self.window.width)
                || bottom.is_none_or(|value| value > self.window.height)
            {
                return Err(CoreError::InvalidTextBounds {
                    id: text.id.clone(),
                });
            }
        }

        for guide in &self.guides {
            let limit = match guide.axis {
                GuideAxis::Horizontal => self.window.height,
                GuideAxis::Vertical => self.window.width,
            };
            if guide.position > limit {
                return Err(CoreError::GuideOutOfBounds);
            }
        }

        self.output_dimensions()?;
        Ok(())
    }
}

fn require_exactly_one(count: u32, kind: &'static str) -> Result<(), CoreError> {
    match count {
        0 => Err(CoreError::MissingRequiredItem(kind)),
        1 => Ok(()),
        _ => Err(CoreError::DuplicateRequiredItem(kind)),
    }
}

fn item_fits(document: &LayoutDocument, x: u32, y: u32) -> bool {
    let radius = document.icon_size / 2;

    x >= radius
        && y >= radius
        && x.checked_add(radius)
            .is_some_and(|right| right <= document.window.width)
        && y.checked_add(radius)
            .is_some_and(|bottom| bottom <= document.window.height)
}
