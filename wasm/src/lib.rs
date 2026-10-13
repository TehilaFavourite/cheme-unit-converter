//! WebAssembly bindings for `cheme-unit-converter`, used by the web interface.
//!
//! Three functions cross the boundary. Each takes and returns plain
//! JavaScript values and never throws; failures are part of the returned
//! value. The shapes are documented in `docs/adr/0003-wasm-boundary-design.md`
//! and mirrored in `web/src/wasm.ts`.

pub mod api;
pub mod context;

use serde::Serialize;
use wasm_bindgen::prelude::*;

use crate::api::{ApiError, ConvertResponse};

/// Returns every quantity with its units, and every context input.
#[wasm_bindgen]
pub fn catalogue() -> JsValue {
    to_js(&api::catalogue())
}

/// Converts a request object (`ConvertRequest`) and returns a
/// `ConvertResponse`.
#[wasm_bindgen]
pub fn convert(request: JsValue) -> JsValue {
    let response = match serde_wasm_bindgen::from_value::<api::ConvertRequest>(request) {
        Ok(request) => api::convert(&request),
        Err(error) => {
            ConvertResponse::failure(ApiError::internal(format!("malformed request: {error}")))
        }
    };
    to_js(&response)
}

/// Resolves a typed unit name to its display label and quantity.
#[wasm_bindgen(js_name = resolveUnit)]
pub fn resolve_unit(text: &str) -> JsValue {
    to_js(&api::resolve_unit(text))
}

/// Serialises to plain objects (not `Map`s). Serialising these types cannot
/// fail; if it ever did, JavaScript receives `null`, which the typed wrapper
/// reports as an internal error.
fn to_js<T: Serialize>(value: &T) -> JsValue {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .unwrap_or(JsValue::NULL)
}
