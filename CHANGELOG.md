# Changelog

All notable changes to this project are documented here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project uses
[Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - Unreleased

First version.

### Web interface

- A browser interface at <https://tehilafavourite.github.io/cheme-unit-converter/>, built from the same Rust
  code as WebAssembly. It converts as you type, reveals context fields only when the library reports them
  missing, marks supplied context a conversion does not use, and keeps every conversion in a shareable URL.
- `cheme-unit-converter-wasm`, a binding crate exposing `catalogue`, `convert` and `resolveUnit` with typed
  errors. A parity test checks its output against the CLI on 53 cases.
- `format_value` and `SignificantFigures` in the library, shared by the CLI and the web interface. CLI
  output is unchanged.
- CI covers the web build (wasm-bindgen-test, tsc, ESLint, Prettier, Vitest, Playwright with axe, and a
  gzip size budget). GitHub Pages deploys after CI passes on `main`.

### Command-line tool

- `convert <value> <from-unit> <to-unit> [options]`, `units`, `--help`, and
  `--version`.
- 138 units across 14 quantities: temperature, temperature difference,
  pressure (absolute and gauge), flow (mass, volumetric, molar, standard gas),
  composition, density (including SG and °API), viscosity (dynamic and
  kinematic), energy, power, heat transfer coefficient, thermal conductivity,
  heat flux, specific heat capacity, and specific enthalpy.
- Unit names match regardless of case and punctuation, so `W/m2K` matches
  `W/(m²·K)`. `mPa`, `mJ`, and `mW` are refused, since they would otherwise read
  as MPa, MJ, and MW.
- Context is never assumed. Density, molar masses, reference density,
  atmospheric pressure, standard reference conditions, actual gas conditions,
  and the compressibility factor Z are supplied as options, and an error names
  the option that is missing.
- Results show up to 10 significant figures by default, or exactly n with
  `--sig-figs n`.
- Options may appear anywhere after `convert`. Exit status is 0 on success, 1
  for a failed conversion, and 2 for invalid usage.

### Library

- `convert`, `parse_unit`, `Unit` (with `FromStr` and `Display`), and a typed
  newtype for each quantity.
- `ConversionContext` with `with_*` builder methods.
- `ConversionError` and `ParseUnitError`, both implementing
  `std::error::Error`.
- Public enums and `ConversionContext` are `#[non_exhaustive]`, so units and
  context fields can be added without breaking callers.
