# 0002. Build the interface with plain TypeScript and CSS, without a UI framework

- Status: Accepted
- Date: 2026-10-13

## Context

The interface is a single screen:
- a value
- a quantity selector
- two unit fields
- a swap button
- optional significant figures
- context fields that appear when Rust asks for them
- a result
- a reference table

All engineering logic lives in Rust ([ADR 0001](0001-webassembly-over-server.md)). The TypeScript layer only
holds state, mirrors it to the URL, calls Wasm and renders. We want a small download, few moving parts, and code
a reviewer can read in one sitting.

## Decision

Use Vite with strict TypeScript and plain CSS. State is a single plain object. Rendering is done by small
functions in `web/src/ui/`, each building or updating one part of the DOM. CSS custom properties in
`styles/tokens.css` define the design tokens.

## Alternatives considered

### React (or Preact, Svelte, Vue)

- For: declarative rendering; a familiar component model; a large ecosystem.
- Against:
  - A runtime and a build plugin, plus their upgrade churn, for one screen with about ten inputs.
  - React's controlled inputs interact awkwardly with `<datalist>`, caret position during live
    conversion, and `history.replaceState` on every keystroke.
  - The bundle would be several times larger than the application code.

### Leptos or Yew (Rust UI frameworks compiled to Wasm)

- For: one language end to end; types shared with the library without a JSON boundary.
- Against:
  - The UI would be built into the Wasm binary, which grows much larger than a binding crate exposing
    three functions.
  - These frameworks move faster than the library and would pin the web build to their release cadence and
    toolchain requirements.
  - DOM and accessibility work (labels, live regions, focus) is less direct than in plain HTML.
  - Contributors would need to know the framework as well as Rust.

## Consequences

- No runtime dependencies in the shipped JavaScript. Vite, TypeScript, Vitest, ESLint, Prettier and Playwright
  are development-only.
- Re-rendering is explicit. With a fixed set of fields, the render functions update existing elements
  instead of rebuilding them, so focus and caret position survive live conversion.
- Without a framework to enforce structure, we keep it by convention:
  - `wasm.ts` is the only module that touches the bindings.
  - `state.ts` is pure and has no DOM or Wasm access, so Vitest can test it in Node.
  - Each `ui/` module renders one component.
