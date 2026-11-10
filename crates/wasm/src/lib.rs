//! Thin wasm-bindgen wrapper. No logic here: only (de)serialisation around `core-logic`.

use api_types::CreateItem;
use wasm_bindgen::prelude::*;

/// Returns `undefined` on success or an array of `FieldError` on failure.
#[wasm_bindgen]
pub fn validate_create_item(input: JsValue) -> Result<JsValue, JsValue> {
    let input: CreateItem = serde_wasm_bindgen::from_value(input)?;
    match core_logic::validate_create_item(&input) {
        Ok(_) => Ok(JsValue::UNDEFINED),
        Err(errors) => Ok(serde_wasm_bindgen::to_value(&errors)?),
    }
}
