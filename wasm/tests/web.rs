//! Tests of the exported functions as JavaScript sees them, run inside a
//! WebAssembly runtime with `cargo test --target wasm32-unknown-unknown`.
#![cfg(target_arch = "wasm32")]

use cheme_unit_converter_wasm::{catalogue, convert, resolve_unit};
use serde_json::{Value, json};
use wasm_bindgen::JsValue;
use wasm_bindgen_test::wasm_bindgen_test;

/// Converts a JavaScript value back into JSON for assertions.
fn json_of(value: JsValue) -> Value {
    match serde_wasm_bindgen::from_value(value) {
        Ok(json) => json,
        Err(error) => panic!("export returned a value that is not plain JSON: {error}"),
    }
}

/// Calls `convert` with a request written as JSON.
fn convert_json(request: &Value) -> Value {
    let serializer = serde_wasm_bindgen::Serializer::json_compatible();
    match serde::Serialize::serialize(request, &serializer) {
        Ok(request) => json_of(convert(request)),
        Err(error) => panic!("test request could not be converted: {error}"),
    }
}

#[wasm_bindgen_test]
fn catalogue_lists_every_quantity_unit_and_context_item() {
    let catalogue = json_of(catalogue());
    let Some(quantities) = catalogue["quantities"].as_array() else {
        panic!("quantities should be an array: {catalogue}");
    };
    assert_eq!(quantities.len(), 14);
    let units: usize = quantities
        .iter()
        .filter_map(|quantity| quantity["units"].as_array().map(Vec::len))
        .sum();
    assert_eq!(units, cheme_unit_converter::Unit::all().len());
    assert_eq!(quantities[0]["key"], "temperature");
    assert_eq!(quantities[0]["units"][0]["label"], "°C");
    assert!(quantities[0]["units"][0]["aliases"].is_array());

    let Some(context) = catalogue["context"].as_array() else {
        panic!("context should be an array: {catalogue}");
    };
    assert_eq!(context.len(), 10);
    assert_eq!(context[0]["key"], "density");
    assert_eq!(
        context[0]["input"],
        json!({ "kind": "number", "unit": "kg/m³" })
    );
    assert_eq!(catalogue["maxSignificantFigures"], 17);
}

#[wasm_bindgen_test]
fn successful_conversion_has_the_documented_shape() {
    let response = convert_json(&json!({
        "value": "1", "from": "m3/s", "to": "kg/h", "sigFigs": "4",
        "context": { "density": "1000" }
    }));
    assert_eq!(response["value"].as_f64(), Some(3_600_000.0));
    let Value::Object(mut fields) = response else {
        panic!("response should be an object");
    };
    fields.remove("value");
    assert_eq!(
        Value::Object(fields),
        json!({
            "ok": true,
            "formatted": "3.600e6",
            "from": "m³/s",
            "to": "kg/h",
            "quantity": "flowRate",
            "used": ["density"]
        })
    );
}

#[wasm_bindgen_test]
fn missing_context_is_a_typed_error() {
    let response = convert_json(&json!({ "value": "30", "from": "psig", "to": "bar" }));
    assert_eq!(
        response,
        json!({
            "ok": false,
            "error": {
                "kind": "missingContext",
                "message": "converting between gauge and absolute pressure requires the local atmospheric pressure",
                "field": "atmosphericPressure",
                "missing": "atmosphericPressure"
            }
        })
    );
}

#[wasm_bindgen_test]
fn invalid_input_is_reported_against_its_field() {
    let invalid_number = convert_json(&json!({ "value": "abc", "from": "C", "to": "F" }));
    assert_eq!(invalid_number["error"]["kind"], "invalidNumber");
    assert_eq!(invalid_number["error"]["field"], "value");

    let unknown_unit = convert_json(&json!({ "value": "1", "from": "C", "to": "furlong" }));
    assert_eq!(unknown_unit["error"]["kind"], "unrecognizedUnit");
    assert_eq!(unknown_unit["error"]["field"], "to");

    let bad_context = convert_json(&json!({
        "value": "1", "from": "psig", "to": "psi",
        "context": { "atmosphericPressure": { "value": "x", "unit": "kPa" } }
    }));
    assert_eq!(bad_context["error"]["kind"], "invalidNumber");
    assert_eq!(bad_context["error"]["field"], "atmosphericPressure");
}

#[wasm_bindgen_test]
fn malformed_requests_do_not_throw() {
    let response = json_of(convert(JsValue::from_str("not an object")));
    assert_eq!(response["ok"], false);
    assert_eq!(response["error"]["kind"], "internal");

    let missing_field = convert_json(&json!({ "from": "C", "to": "F" }));
    assert_eq!(missing_field["ok"], false);
    assert_eq!(missing_field["error"]["kind"], "internal");
}

#[wasm_bindgen_test]
fn resolves_typed_aliases() {
    assert_eq!(
        json_of(resolve_unit("degC")),
        json!({ "ok": true, "label": "°C", "quantity": "temperature" })
    );
    let unresolved = json_of(resolve_unit("furlong"));
    assert_eq!(unresolved["ok"], false);
    assert_eq!(unresolved["error"]["kind"], "unrecognizedUnit");
}
