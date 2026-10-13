# Chemical Engineering Unit Converter

A dependency-free Rust command-line tool and library for converting units
commonly used in chemical engineering. Supported measurements include
temperature, absolute pressure, mass and volumetric flow, and concentration.

## Contents

- [Features](#features)
- [Requirements](#requirements)
- [Build and test](#build-and-test)
- [Command-line usage](#command-line-usage)
- [Units and aliases](#units-and-aliases)
- [Context-dependent conversions](#context-dependent-conversions)
- [Engineering assumptions](#engineering-assumptions)
- [Library usage](#library-usage)
- [Project layout](#project-layout)

## Features

- Converts temperature (°C, °F, K), pressure (bar, psi, atm, kPa), flow rate
  (kg/h, m³/s, US GPM, lb/min), and concentration (mol/L, wt%, ppm, mg/L).
- Uses enums to distinguish units and newtypes to keep quantities separate.
  For example, a `Temperature` cannot be passed where a `Pressure` is expected.
- Converts through canonical internal units: kelvin, pascals, kilograms per
  second, and solute mass fraction.
- Checks for invalid numeric inputs, below-absolute-zero temperatures,
  negative absolute pressure/flow/concentration, invalid context, and
  incompatible quantities.
- Requires density and molar mass where the conversion needs those properties.
- Prints results with up to 10 digits after the decimal point.

## Requirements

- Rust 1.85 or newer (edition 2024).
- No third-party dependencies.

## Build and test

From this directory:

```sh
cargo build
cargo test
cargo clippy --all-targets -- -D warnings
```

## Command-line usage

```text
cheme-unit-converter convert <value> <from-unit> <to-unit> [options]
cheme-unit-converter --help
```

Use Cargo to run the command from the source checkout:

```sh
cargo run -- convert 25 C F
cargo run -- convert 2.5 bar psi
cargo run -- convert 1200 kg/h lb/min
cargo run -- convert 1.2 mol/L mg/L --density 1000 --molar-mass 58.44
```

After building, invoke the executable directly:

```sh
./target/debug/cheme-unit-converter convert 25 C F
```

Options can be supplied in either order:

```sh
cargo run -- convert 1 m3/s kg/h --density 1000
cargo run -- convert 1.2 mol/L wt% --molar-mass 58.44 --density 1000
```

For concentration conversions that require both values, `--density` and
`--molar-mass` may appear in either order. Each option accepts one finite
positive number and may only be specified once. Errors are written to standard
error and the process exits with a non-zero status.

## Units and aliases

Unit names are case-insensitive. The parser accepts the following names:

| Quantity | Preferred label | Accepted inputs |
| --- | --- | --- |
| Temperature | °C | `C`, `°C`, `celsius` |
| Temperature | °F | `F`, `°F`, `fahrenheit` |
| Temperature | K | `K`, `kelvin` |
| Pressure | bar | `bar` |
| Pressure | psi | `psi` |
| Pressure | atm | `atm` |
| Pressure | kPa | `kPa` |
| Mass flow | kg/h | `kg/h`, `kg/hr` |
| Volume flow | m³/s | `m3/s`, `m^3/s`, `m³/s` |
| Volume flow | GPM | `GPM` (US gallons per minute) |
| Mass flow | lb/min | `lb/min`, `lbs/min` |
| Concentration | mol/L | `mol/L`, `mol/litre`, `mol/liter` |
| Concentration | wt% | `wt%`, `wtpercent` |
| Concentration | ppm | `ppm` |
| Concentration | mg/L | `mg/L`, `mg/litre`, `mg/liter` |

## Context-dependent conversions

Some units describe quantities on different bases. Those conversions cannot be
determined from the input value alone, so provide the required physical
properties explicitly:

| Option | Required when | Units |
| --- | --- | --- |
| `--density <value>` | Converting to or from a volumetric flow rate when the other unit is a mass flow rate; converting concentration involving mol/L or mg/L | kg/m³ |
| `--molar-mass <value>` | Converting concentration involving mol/L | g/mol |

Examples:

```sh
# 1 m³/s of a fluid with density 1000 kg/m³ -> kg/h
cargo run -- convert 1 m3/s kg/h --density 1000

# 1.2 mol/L of a solute with molar mass 58.44 g/mol
# in a solution with density 1000 kg/m³ -> mg/L
cargo run -- convert 1.2 mol/L mg/L --density 1000 --molar-mass 58.44
```

Density and molar mass must be finite and greater than zero. They are not
guessed or defaulted by the converter.

## Engineering assumptions

- Pressure units are **absolute**. Convert gauge pressure to absolute pressure
  before using the tool.
- `GPM` means **US** gallons per minute, not Imperial gallons per minute.
- `ppm` is interpreted as a mass fraction (mg/kg); `wt%` is mass percent.
  Their conversion does not require density.
- Converting mg/L requires solution density to relate solution volume to mass.
  Converting mol/L requires both density and solute molar mass.
- Concentrations represent solute mass divided by total solution mass.
  Fractions above 100% are rejected.
- Flow rates and concentrations are non-negative. Temperature may be negative
  on the Celsius or Fahrenheit scale, but not below absolute zero.
- Calculations use `f64`; output is rounded for display to at most 10 decimal
  places. The displayed value is not intended to imply more measurement
  accuracy than the supplied data.

## Library usage

The conversion logic is also available as a Rust library. Add a path
dependency if using it from another local project:

```toml
[dependencies]
cheme-unit-converter = { path = "../cheme-unit-converter" }
```

Then parse units and call `convert`:

```rust
use cheme_unit_converter::{ConversionContext, convert, parse_unit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let value = convert(
        25.0,
        parse_unit("C")?,
        parse_unit("F")?,
        ConversionContext::default(),
    )?;
    println!("{value} °F");
    Ok(())
}
```

The canonical unit types (`Temperature`, `Pressure`, `FlowRate`, and
`Concentration`) are also public for code that needs to construct and convert
individual quantities directly. See the Rust API documentation generated by
`cargo doc --open`.

## Project layout

```text
cheme-unit-converter/
├── Cargo.toml
├── README.md
└── src/
    ├── lib.rs    # Unit types, validation, conversions, and unit tests
    └── main.rs   # Command-line argument parsing and output
```

This directory is a standalone Git repository, separate from the surrounding
Rust workspace, so it can be committed and pushed independently.
# cheme-unit-converter
