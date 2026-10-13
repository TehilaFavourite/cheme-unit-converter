# 0003. Wasm boundary: three JSON functions, typed errors, Rust-side parsing

- Status: Accepted
- Date: 2026-10-13

## Context

The interface calls the Rust library through WebAssembly ([ADR 0001](0001-webassembly-over-server.md)). The
boundary decides how much logic stays in Rust. It must be:

- **complete**: the TypeScript side never needs unit lists, parsing rules, validation or messages of its own;
- **typed**: the interface reacts to errors such as missing context by revealing a specific field, so errors
  must be structured data, not just strings;
- **stable and small**: few functions with documented shapes that change rarely;
- **in step with the CLI**: the web interface and the CLI give identical results for identical input.

## Decision

### Crate layout

A separate crate, `wasm/` (`cheme-unit-converter-wasm`, `cdylib` + `rlib`), depends on the root crate and on
`wasm-bindgen`, `serde` and `serde-wasm-bindgen`. The root crate stays dependency-free. Handlers are plain
Rust functions in `wasm/src/api.rs`, testable natively. `wasm/src/lib.rs` only wraps them with
`#[wasm_bindgen]`.

### Exports

None of them throws: every failure is a value.

| Export | Returns |
| --- | --- |
| `catalogue()` | Every quantity with its units (label and aliases), and every context item with its description, input kind and query-string name |
| `convert(request)` | `{ ok: true, value, formatted, from, to, quantity, used }` or `{ ok: false, error: { kind, message, field, missing? } }` |
| `resolveUnit(text)` | `{ ok: true, label, quantity }` or `{ ok: false, message }` |

The exact TypeScript shapes are mirrored in `web/src/wasm.ts`.

### Rules

1. **Numbers cross the boundary as text.** Value, context values and significant figures are sent as the user
   typed them, and Rust parses them with the same `str::parse::<f64>` the CLI uses. The web interface therefore
   accepts and rejects exactly what the CLI does, including `1e3` and the error for `abc`.
2. **Errors are typed.** `kind` is a stable camelCase discriminant (`missingContext`, `invalidNumber`,
   `incompatibleUnits`, ...).
   - `field` names the input at fault, so the interface can show the message next to it.
   - `missing` names the context item to reveal.
   - `message` is the library's own text: the `ConversionError` and `ParseUnitError` `Display` output.
3. **Context is built in Rust, in the CLI's order, and units are never assumed.** Atmospheric pressure is built first so that gauge context
   pressures can use it, and it accepts only absolute units. A gauge context pressure without an atmospheric
   pressure reports `missing: "atmosphericPressure"` with `field` set to that pressure. A context value
   typed without a unit is an `emptyInput` error for that field, so the interface never preselects one.
4. **`used` makes stale context visible.**
   - After a successful conversion, Rust removes each supplied context item in turn and converts again. An item
     is used exactly when its removal causes an error, because the library never falls back to a default.
   - The interface marks filled-in but unused fields, so no value can affect a result without being shown.
5. **Formatting lives in the library.** `format_value` and `SignificantFigures` move from the CLI into the root
   crate, and both front ends call them.
6. **Parity is tested.**
   - A native test in `wasm/` builds the real CLI binary, runs it on at least 30 cases across all fourteen
     quantities, and asserts that its output line equals the line assembled from the Wasm response.
   - Error cases compare messages.

### Build tooling: wasm-bindgen-cli rather than wasm-pack

We call `cargo build --target wasm32-unknown-unknown`, `wasm-bindgen --target web` and `wasm-opt -Oz` from a
short script (`scripts/build-wasm.sh`).

- `wasm-bindgen-cli` must match the `wasm-bindgen` crate version exactly. With both pinned to the same
  `=x.y.z` (in `wasm/Cargo.toml` and in CI), a mismatch is impossible to miss.
- The same package provides `wasm-bindgen-test-runner`, so the binding tests run through
  `cargo test --target wasm32-unknown-unknown` with no extra tool.
- wasm-pack would add a wrapper that downloads binaries at build time, plus a version of its own to track.
  We use none of its packaging features, because we do not publish to npm.

## Alternatives considered

- **Exporting Rust types directly with `#[wasm_bindgen]` structs and enums.**
  - wasm-bindgen cannot export enums carrying data, so errors would flatten into strings or class handles that
    need manual freeing.
  - JSON values are self-describing and easy to assert on in tests.
- **TypeScript parses numbers and sends `f64`.** The web interface would drift from the CLI on edge cases
  (`Number("")` is `0`, `Number("0x10")` is `16`) and would need its own validation messages.
- **Returning only a message string on error.** The interface could not tell missing context from a bad
  unit without parsing English.
- **Computing every context requirement up front.** That would need placeholder values passed through the
  library, which contradicts "context is never assumed". Requirements arrive one at a time, as the library
  reports them, and the interface reveals fields progressively.

## Consequences

- The TypeScript side holds no engineering knowledge. Adding a unit or context item to the library flows
  through `catalogue()` without TypeScript changes, unless a new context input kind appears.
- JSON conversion costs a little per call, which is negligible against keystroke rates.
- The JSON shapes are a contract. Changing them means updating `web/src/wasm.ts`, the wasm-bindgen tests and
  this ADR together.
- Running every conversion up to ten more times to compute `used` is negligible at microseconds per
  conversion.
