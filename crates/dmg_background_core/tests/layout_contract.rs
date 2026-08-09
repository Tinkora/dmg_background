use dmg_background_core::{
    CoreError, Guide, GuideAxis, ItemKind, LayoutDocument, LayoutItem, MAX_RETINA_PIXELS,
    SCHEMA_URI, TextElement, Window, layout_schema_json, retina_dimensions,
};

fn valid_document() -> LayoutDocument {
    LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Example",
    )
    .expect("fixture window should be valid")
}

#[test]
fn default_layout_is_valid_and_serializable() {
    let document = valid_document();
    document.validate().expect("default layout should validate");

    let json = serde_json::to_value(&document).expect("layout should serialize");
    assert_eq!(json["$schema"], SCHEMA_URI);
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["coordinate_space"], "finder_content_logical_points");
    assert_eq!(json["background"]["one_x"], ".background/background.png");
    assert_eq!(json["background"]["two_x"], ".background/background@2x.png");
    assert_eq!(json["items"][0]["label"], "Example.app");
}

#[test]
fn default_layout_uses_the_volume_name_for_the_application_label() {
    let document = LayoutDocument::app_to_applications(
        Window {
            width: 800,
            height: 450,
        },
        "Tinkora",
    )
    .expect("fixture window should be valid");
    let application = document
        .items
        .iter()
        .find(|item| item.kind == ItemKind::Application)
        .expect("application item should exist");

    assert_eq!(application.label, "Tinkora.app");
}

#[test]
fn bundled_schema_is_valid_json_with_version_one() {
    let schema: serde_json::Value =
        serde_json::from_str(layout_schema_json()).expect("schema should be valid JSON");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    assert_eq!(schema["properties"]["schema_version"]["const"], 1);
}

#[test]
fn schema_uri_and_version_must_match_contract() {
    let mut wrong_uri = valid_document();
    wrong_uri.schema_uri = "https://example.invalid/layout.json".to_owned();
    assert_eq!(
        wrong_uri.validate(),
        Err(CoreError::UnsupportedSchemaUri(
            "https://example.invalid/layout.json".to_owned()
        ))
    );

    let mut wrong_version = valid_document();
    wrong_version.schema_version = 2;
    assert_eq!(
        wrong_version.validate(),
        Err(CoreError::UnsupportedSchemaVersion(2))
    );
}

#[test]
fn default_constructor_rejects_invalid_windows_without_overflowing() {
    assert_eq!(
        LayoutDocument::app_to_applications(
            Window {
                width: u32::MAX,
                height: 450,
            },
            "Invalid",
        ),
        Err(CoreError::InvalidWindowSize {
            width: u32::MAX,
            height: 450,
        })
    );
}

#[test]
fn required_items_must_exist_exactly_once() {
    let mut missing = valid_document();
    missing
        .items
        .retain(|item| item.kind != ItemKind::ApplicationsAlias);
    assert_eq!(
        missing.validate(),
        Err(CoreError::MissingRequiredItem("applications_alias"))
    );

    let mut duplicate = valid_document();
    duplicate.items.push(LayoutItem {
        id: "second-app".to_owned(),
        kind: ItemKind::Application,
        x: 400,
        y: 225,
        label: "Other.app".to_owned(),
    });
    assert_eq!(
        duplicate.validate(),
        Err(CoreError::DuplicateRequiredItem("application"))
    );
}

#[test]
fn item_ids_are_unique_and_icon_rectangles_stay_inside_window() {
    let mut duplicate_id = valid_document();
    duplicate_id.items[1].id = duplicate_id.items[0].id.clone();
    assert_eq!(
        duplicate_id.validate(),
        Err(CoreError::DuplicateItemId("app".to_owned()))
    );

    let mut out_of_bounds = valid_document();
    out_of_bounds.items[0].x = 1;
    assert_eq!(
        out_of_bounds.validate(),
        Err(CoreError::ItemOutOfBounds {
            id: "app".to_owned()
        })
    );
}

#[test]
fn text_and_guide_geometry_must_stay_inside_window() {
    let mut document = valid_document();
    document.texts.push(TextElement {
        id: "title".to_owned(),
        x: 100,
        y: 40,
        width: 600,
        height: 60,
        font_size: 32,
        content: "Install Example".to_owned(),
    });
    document.guides.push(Guide {
        axis: GuideAxis::Vertical,
        position: 400,
    });
    document
        .validate()
        .expect("valid text and guide should pass");

    document.texts[0].width = 701;
    assert_eq!(
        document.validate(),
        Err(CoreError::InvalidTextBounds {
            id: "title".to_owned()
        })
    );

    let mut invalid_guide = valid_document();
    invalid_guide.guides.push(Guide {
        axis: GuideAxis::Horizontal,
        position: 451,
    });
    assert_eq!(invalid_guide.validate(), Err(CoreError::GuideOutOfBounds));
}

#[test]
fn icon_size_must_be_even_for_symmetric_center_coordinates() {
    let mut document = valid_document();
    document.icon_size = 127;
    assert_eq!(document.validate(), Err(CoreError::InvalidIconSize));
}

#[test]
fn retina_dimensions_are_exact_and_overflow_is_rejected() {
    let dimensions = retina_dimensions(800, 450).expect("normal dimensions should work");
    assert_eq!(dimensions.one_x.width, 800);
    assert_eq!(dimensions.one_x.height, 450);
    assert_eq!(dimensions.two_x.width, 1600);
    assert_eq!(dimensions.two_x.height, 900);

    assert_eq!(
        retina_dimensions(u32::MAX, 450),
        Err(CoreError::OutputDimensionOverflow)
    );
}

#[test]
fn retina_output_enforces_the_pixel_budget() {
    let boundary = retina_dimensions(4096, 1024).expect("the exact pixel budget should pass");
    assert_eq!(
        u64::from(boundary.two_x.width) * u64::from(boundary.two_x.height),
        MAX_RETINA_PIXELS
    );

    assert_eq!(
        retina_dimensions(4096, 1025),
        Err(CoreError::OutputPixelBudgetExceeded {
            pixels: 16_793_600,
            max_pixels: MAX_RETINA_PIXELS,
        })
    );

    assert_eq!(
        LayoutDocument::app_to_applications(
            Window {
                width: 4096,
                height: 1025,
            },
            "Too large",
        ),
        Err(CoreError::OutputPixelBudgetExceeded {
            pixels: 16_793_600,
            max_pixels: MAX_RETINA_PIXELS,
        })
    );
}

#[test]
fn required_arrays_cannot_be_omitted() {
    for field in ["texts", "guides"] {
        let mut value = serde_json::to_value(valid_document()).expect("layout should serialize");
        value
            .as_object_mut()
            .expect("layout should be an object")
            .remove(field);

        assert!(
            serde_json::from_value::<LayoutDocument>(value).is_err(),
            "missing {field} must be rejected like the JSON Schema"
        );
    }
}

#[test]
fn unknown_json_fields_are_ignored_when_reading() {
    let mut value = serde_json::to_value(valid_document()).expect("layout should serialize");
    value["future_field"] = serde_json::json!({ "enabled": true });
    value["window"]["future_window_field"] = serde_json::json!("ignored");
    value["items"][0]["future_item_field"] = serde_json::json!(42);

    let decoded: LayoutDocument =
        serde_json::from_value(value).expect("unknown fields should be ignored");
    decoded
        .validate()
        .expect("decoded layout should remain valid");
}
