use dmg_background_core::{CoreError, LayoutDocument, Window, export_contract_zip};
use wasm_bindgen::prelude::*;

/// Converts a CoreError into a JsValue error carrying a stable `code` field.
fn core_err(e: CoreError) -> JsValue {
    let obj = js_sys::Object::new();
    js_sys::Reflect::set(&obj, &"code".into(), &e.code().into()).ok();
    js_sys::Reflect::set(&obj, &"message".into(), &e.to_string().into()).ok();
    obj.into()
}

/// Creates a default "drag app to Applications" layout document in JS.
#[wasm_bindgen]
pub fn create_default_document(
    width: u32,
    height: u32,
    volume_name: &str,
) -> Result<JsValue, JsValue> {
    let window = Window { width, height };
    let doc = LayoutDocument::app_to_applications(window, volume_name).map_err(core_err)?;
    serde_wasm(&doc)
}

/// Deserializes a JS object into a LayoutDocument, validates it, and returns the serialized JSON.
#[wasm_bindgen]
pub fn validate_document(json: &JsValue) -> Result<JsValue, JsValue> {
    let doc: LayoutDocument = serde_wasm_bindgen::from_value(json.clone())
        .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {e}")))?;
    doc.validate().map_err(core_err)?;
    serde_wasm(&doc)
}

/// Exports the contract ZIP from PNG bytes rendered by the JavaScript Canvas layer.
#[wasm_bindgen]
pub fn export_zip(
    document_json: &JsValue,
    background_1x: &[u8],
    background_2x: &[u8],
    preview: &[u8],
) -> Result<Vec<u8>, JsValue> {
    let doc: LayoutDocument = serde_wasm_bindgen::from_value(document_json.clone())
        .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {e}")))?;
    export_contract_zip(&doc, background_1x, background_2x, preview).map_err(core_err)
}

/// Returns the embedded JSON Schema 1 for use by JS-side validators.
#[wasm_bindgen]
pub fn get_layout_schema() -> String {
    dmg_background_core::layout_schema().to_string()
}

/// Computes 1x/2x output dimensions.
#[wasm_bindgen]
pub fn get_output_dimensions(document_json: &JsValue) -> Result<JsValue, JsValue> {
    let doc: LayoutDocument = serde_wasm_bindgen::from_value(document_json.clone())
        .map_err(|e| JsValue::from_str(&format!("Deserialization failed: {e}")))?;
    let dims = doc.output_dimensions().map_err(core_err)?;
    let result = serde_json::json!({
        "one_x": { "width": dims.one_x.width, "height": dims.one_x.height },
        "two_x": { "width": dims.two_x.width, "height": dims.two_x.height }
    });
    serde_wasm(&result)
}

fn serde_wasm<T: serde::Serialize>(value: &T) -> Result<JsValue, JsValue> {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    value
        .serialize(&serializer)
        .map_err(|e| JsValue::from_str(&format!("Serialization failed: {e}")))
}
