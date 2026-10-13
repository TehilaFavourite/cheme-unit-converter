# Architecture decision records

Each record describes one decision: its context, the decision, the alternatives considered, and the
consequences. Once a record is on `main` it is not rewritten. A later record supersedes it instead.

| # | Decision | Status |
| --- | --- | --- |
| [0001](0001-webassembly-over-server.md) | Run the converter in the browser with WebAssembly | Accepted |
| [0002](0002-no-frontend-framework.md) | Build the interface with plain TypeScript and CSS, without a UI framework | Accepted |
| [0003](0003-wasm-boundary-design.md) | Wasm boundary: three JSON functions, typed errors, Rust-side parsing | Accepted |
