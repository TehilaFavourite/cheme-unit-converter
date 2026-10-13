# 0001. Run the converter in the browser with WebAssembly

- Status: Accepted
- Date: 2026-10-13

## Context

The converter is a Rust library with a command-line front end. We want a web interface that students can open
from a link, with results that update as they type. The library is the single source of truth for units,
parsing, validation, context requirements, error messages and number formatting. Whatever the web interface
does, it must not reimplement any of that, and its results must match the CLI exactly.

The library has no dependencies, does no I/O and keeps no state. A conversion takes microseconds.

## Decision

Compile the library to WebAssembly through a separate binding crate (`wasm/`), and serve the interface as static
files from GitHub Pages. The browser runs the same Rust code the CLI runs. No application server exists.

## Alternatives considered

### An Axum (or other Rust) HTTP server

The library would sit behind a JSON endpoint, with a thin page calling it.

- For: no Wasm toolchain; the server is ordinary Rust; the page bundle is smaller.
- Against:
  - Every keystroke becomes a network round trip, so live conversion feels slow on poor connections, and
    stops working offline.
  - It needs hosting, a domain, TLS, uptime and patching, all for a computation that is pure and takes
    microseconds.
  - It adds an attack surface (input parsing on a public endpoint) and operating costs that a student tool
    cannot justify.
  - Sharing a link only works while the server is up.

### Reimplementing the conversions in TypeScript

Rejected outright. It would create a second source of truth that could drift from the CLI, which is the very
thing the parity test exists to prevent.

## Consequences

- Hosting is free and static (GitHub Pages), and conversion latency is the cost of a function call.
- The page must download and instantiate a Wasm module before the first conversion. We keep that small with
  a size-optimised release profile (`opt-level = "z"`, LTO, `wasm-opt`) and report the gzipped size in CI.
- The build needs Rust tooling (`wasm32-unknown-unknown`, `wasm-bindgen-cli`, `wasm-opt`) alongside Node.
  This is confined to the `web` CI job and the `wasm/` crate, so the root crate stays dependency-free and its
  MSRV job is unaffected.
- The binding crate is a new boundary to keep stable. See [ADR 0003](0003-wasm-boundary-design.md).
