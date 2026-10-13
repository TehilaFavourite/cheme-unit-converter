# Chemical Engineering Unit Converter

[![CI](https://github.com/TehilaFavourite/cheme-unit-converter/actions/workflows/ci.yml/badge.svg)](https://github.com/TehilaFavourite/cheme-unit-converter/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A dependency-free Rust command-line tool and library for converting units
commonly used in chemical engineering coursework and lab work: temperature
and temperature differences, absolute and gauge pressure, mass/volumetric/
molar/standard-gas flow, composition, density, viscosity, energy, power,
and heat-transfer properties (heat transfer coefficient, thermal
conductivity, heat flux, specific heat, specific enthalpy).

## Contents

- [Features](#features)
- [Requirements](#requirements)
- [Build and test](#build-and-test)
- [Command-line usage](#command-line-usage)
- [Units](#units)
- [Context-dependent conversions](#context-dependent-conversions)
- [Engineering assumptions](#engineering-assumptions)
- [Library usage](#library-usage)
- [Web interface](#web-interface)
- [Project layout](#project-layout)
- [License](#license)

## Features

- 138 units across fourteen quantities. Run `cheme-unit-converter units`
  to list them with every accepted spelling.
- Temperature readings (°C) and temperature differences (Δ°C) are separate
  quantities, so a 10 °C rise converts to 18 Δ°F, not 50 °F.
- Enums distinguish units, and newtypes keep quantities separate. For example,
  a `Temperature` cannot be passed where a `Pressure` is expected.
- **Context is never assumed.** If a conversion depends on a material
  property, reference condition, or convention (density, molar mass,
  atmospheric pressure, standard conditions, SG reference, compressibility),
  you must supply it.
  If it is missing, the error names what is needed, why, and which option
  provides it.
- Energy and power are separate quantities, so kWh → kW is refused rather
  than silently treated as a per-hour rate.
- Within a basis, a conversion only needs the unit factors. L/min → GPM needs no
  density, and mol/L → mmol/L needs no molar mass.
- Results show up to 10 significant figures by default. `--sig-figs n` rounds
  to exactly n. Very large and very small results use scientific notation.

## Requirements

- Rust 1.85 or newer (edition 2024). CI tests both 1.85 and the latest stable
  release on Linux, macOS and Windows.
- No third-party dependencies.

## Build and test

From this directory:

```sh
cargo build
cargo test                                  # unit, CLI, round-trip, and doc tests
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

`cargo install --path .` puts `cheme-unit-converter` on your `PATH`.

## Command-line usage

```text
cheme-unit-converter convert <value> <from-unit> <to-unit> [options]
cheme-unit-converter units
cheme-unit-converter --help | --version
```

Examples (via `cargo run --`):

```sh
cargo run -- convert 25 C F                    # 25 °C = 77 °F
cargo run -- convert 10 dC dF                  # 10 Δ°C = 18 Δ°F
cargo run -- convert 1 GPM L/min               # no density needed
cargo run -- convert 30 psig bar --atm-pressure 101.325 kPa
cargo run -- convert 100 kmol/h Sm3/h --ref-temp 0 C --ref-pressure 1 atm
cargo run -- convert 1.2 mol/L wt% --density 1045 --molar-mass 58.44 --sig-figs 3
cargo run -- convert 42.5 cP cSt --density 850
cargo run -- convert 0.85 SG API
cargo run -- convert 500 TR kW                 # refrigeration duty
cargo run -- convert 150 "BTU/(h·ft2·F)" W/m2K # 851.7 W/(m²·K)
cargo run -- convert 40.65 kJ/mol kJ/kg --molar-mass 18.015
```

When something is missing, the tool explains what to add:

```text
$ cheme-unit-converter convert 30 psig bar
Error: converting between gauge and absolute pressure requires the local atmospheric pressure
Hint: supply --atm-pressure <value> <unit>, for example --atm-pressure 101.325 kPa
```

Options may appear in any order, before or after the value and units. Each may
be given only once. Errors go to standard error. Exit status:

| Status | Meaning |
| --- | --- |
| 0 | Success |
| 1 | The conversion failed (missing context, out-of-range value, incompatible units) |
| 2 | Invalid usage (unknown command, option, or unit; malformed number) |

## Units

Matching ignores case, spaces, `·`, `*`, `.`, `-`, `^`, `°` and
parentheses. You can type `²` and `³` as `2` and `3`, and `hr` as `h`. So
`W/m2K`, `W/(m²·K)` and `w/m^2.k` are the same unit, as are `C` and `°C`,
`mmH2O` and `mmH₂O`, and `Pa.s` and `Pa·s`. The table shows preferred labels.
`cheme-unit-converter units` lists the extra spellings each unit accepts, such
as `dC`, `CFM` and `Nm3/h`. Quote units that contain parentheses in the shell:
`"BTU/(h·ft2·F)"`.

| Quantity | Units |
| --- | --- |
| Temperature | °C, °F, K, °R |
| Temperature difference | Δ°C (`dC`), Δ°F (`dF`), ΔK (`dK`), Δ°R (`dR`) |
| Pressure (absolute) | Pa, kPa, MPa, bar, mbar, atm, psi (`psia`), torr, mmHg, inHg, mmH₂O, inH₂O, ftH₂O, mH₂O, kgf/cm² |
| Pressure (gauge) | kPag, barg, psig |
| Mass flow | kg/s, kg/h, g/s, t/h, lb/s, lb/min, lb/h |
| Volumetric flow | m³/s, m³/h, L/s, L/min, mL/min, GPM (US), ft³/min (`CFM`), bbl/d |
| Molar flow | mol/s, kmol/h, lbmol/h |
| Standard gas flow | Sm³/h (`Nm3/h`), SCFM, MMSCFD, SLPM, SCCM |
| Mass fraction | kg/kg, wt%, ppm (mass), ppb (mass) |
| Mass per volume | mg/L, g/L |
| Molar concentration | mol/L, mmol/L |
| Mole fraction | mol/mol (`x`), mol%, ppm(mol) |
| Molality | mol/kg (solvent) |
| Density | kg/m³, g/cm³, g/mL, kg/L, lb/ft³, lb/gal (US), SG, °API |
| Dynamic viscosity | Pa·s, mPa·s, cP, P, lb/(ft·s), lb/(ft·h) |
| Kinematic viscosity | m²/s, mm²/s, cSt, St, ft²/s |
| Energy | J, kJ, MJ, GJ, kWh, cal, kcal, cal_IT, kcal_IT, BTU, MMBTU, ft·lbf |
| Power / heat duty | W, kW, MW, kJ/h, MJ/h, hp, hp(metric) (`PS`), BTU/h, MMBTU/h, kcal/h, TR (tons of refrigeration) |
| Heat transfer coefficient | W/(m²·K), kW/(m²·K), BTU/(h·ft²·°F), kcal/(h·m²·°C), cal/(s·cm²·°C) |
| Thermal conductivity | W/(m·K), BTU/(h·ft·°F), kcal/(h·m·°C), cal/(s·cm·°C) |
| Heat flux | W/m², kW/m², BTU/(h·ft²), kcal/(h·m²) |
| Specific heat (mass) | J/(kg·K), kJ/(kg·K), J/(g·K), BTU/(lb·°F), cal/(g·°C), kcal/(kg·°C) |
| Specific heat (molar) | J/(mol·K), kJ/(kmol·K), BTU/(lbmol·°F), cal/(mol·K) |
| Specific enthalpy (mass) | J/kg, kJ/kg, J/g, BTU/lb, cal/g, kcal/kg |
| Specific enthalpy (molar) | J/mol, kJ/mol, kJ/kmol, BTU/lbmol, cal/mol, kcal/mol |

`mPa`, `mJ` and `mW` are rejected on purpose. Because matching is
case-insensitive, they would otherwise be read as MPa, MJ and MW, which are
10⁹ times larger.

## Context-dependent conversions

Some conversions cross between bases that cannot be related by unit factors
alone. Supply the needed properties as options:

| Option | Needed when converting | Unit |
| --- | --- | --- |
| `--density <value>` | mass ↔ volumetric flow; volumetric ↔ molar flow; dynamic ↔ kinematic viscosity; mg/L, g/L, mol/L ↔ fractions or molality | kg/m³ |
| `--molar-mass <value>` | mass or volumetric ↔ molar or standard gas flow; mol/L ↔ mass-based units; mole fraction or molality ↔ mass-based units; mass ↔ molar specific heat or enthalpy | g/mol |
| `--solvent-molar-mass <value>` | mole fraction ↔ mass fraction or molality | g/mol |
| `--ref-density <value>` | SG or °API ↔ absolute density | kg/m³ |
| `--atm-pressure <value> <unit>` | gauge ↔ absolute pressure | any absolute pressure unit |
| `--ref-temp <value> <unit>` | standard gas flow ↔ anything outside its basis | any temperature unit |
| `--ref-pressure <value> <unit>` | standard gas flow ↔ anything outside its basis | any pressure unit |
| `--temp <value> <unit>` | standard ↔ actual gas volume flow | any temperature unit |
| `--pressure <value> <unit>` | standard ↔ actual gas volume flow | any pressure unit; gauge units also need `--atm-pressure` |
| `--z <value>` | standard ↔ actual gas volume flow | compressibility factor at the actual conditions; 1 for an ideal gas |

A short command-line example for each:

```sh
cargo run -- convert 1 m3/s kg/h --density 1000
cargo run -- convert 1 kmol/h kg/h --molar-mass 18.015
cargo run -- convert 0.3 x wt% --molar-mass 46.07 --solvent-molar-mass 18.015
cargo run -- convert 0.85 SG kg/m3 --ref-density 999.0
cargo run -- convert 500 SCFM kg/h --ref-temp 60 F --ref-pressure 14.696 psi --molar-mass 28.97
cargo run -- convert 1000 Sm3/h m3/h --ref-temp 0 C --ref-pressure 1 atm \
    --temp 150 C --pressure 5 barg --atm-pressure 1.01325 bar --z 1
```

All of these values must be finite and positive. The converter never guesses
or defaults them.

## Engineering assumptions

- **Pressure.** Units without a `g` suffix are absolute. Gauge pressure is
  absolute pressure minus the local atmospheric pressure you supply. Converting
  gauge to gauge (psig → barg) only rescales the difference and needs no
  atmospheric pressure.
- **Manometer units.** mmHg and inHg are the conventional units (mercury at
  0 °C, standard gravity), and torr is exactly 1/760 atm. Water-column units
  (mmH₂O, inH₂O, ftH₂O, mH₂O) use 1000 kg/m³ and standard gravity. The head of
  any other fluid (ρgh) is a calculation, not a unit conversion.
- **Standard gas volume.** "Standard" and "normal" conditions differ between
  industries and countries (0 °C, 15 °C, 20 °C, or 60 °F, at 1 atm or 1 bar), so
  the reference conditions must be given. Sm³/h and Nm³/h are the same unit
  here, defined by whatever reference you supply. Standard volume is
  ideal-gas volume at those conditions, as the industry conventions define it.
  Actual volume is V = Z·n·R·T / P. Pass `--z 1` for an ideal gas, or the
  compressibility factor of the real gas at its actual conditions.
- **Composition.** Everything that is not the solute is treated as the
  solvent. For a multicomponent remainder, pass its average molar mass as
  `--solvent-molar-mass`. `ppm` and `ppb` are mass-based (mg/kg and µg/kg); use
  `ppm(mol)` for a mole basis. For ideal gases, volume fraction equals mole
  fraction, so ppmv corresponds to `ppm(mol)` and vol% to `mol%`. For liquids,
  volume fractions are not supported, because they depend on mixing behaviour.
- **Specific gravity.** SG is relative to the reference density you supply.
  °API is defined as 141.5 / SG − 131.5, where SG is measured at 60 °F relative
  to water at 60 °F (about 999.0 kg/m³). To convert °API to absolute density,
  pass that water density as `--ref-density`.
- **Calories.** Two calories are in use, and they differ by about 0.07%. Each
  unit follows the references it is normally taken from:
  - The thermochemical calorie (4.184 J), the chemistry convention, is used by
    `cal`, `kcal`, and the specific heat and enthalpy units (cal/(g·°C),
    kcal/mol, ...).
  - The International Table calorie (4.1868 J), the convention in engineering
    handbooks and steam tables, is used by `cal_IT`, `kcal_IT`, `kcal/h`, and
    the heat transfer coefficient, conductivity and heat-flux units. So
    1 kcal/(h·m²·°C) is 1.163 W/(m²·K).
- **Other energy units.** BTU is the International Table BTU (1055.05585262 J).
  `hp` is mechanical horsepower (550 ft·lbf/s), and `hp(metric)` is 75 kgf·m/s.
- **Heat-transfer units.** The "per degree" in units such as BTU/(lb·°F) and
  W/(m²·K) is a temperature *difference*. A Fahrenheit degree is 5/9 K with no
  offset, so 1 BTU/(lb·°F) is exactly 4.1868 kJ/(kg·K).
- **Signs.** Energy, power, heat flux and specific enthalpy may be negative
  (heat removed, or enthalpy below the reference state). Heat transfer
  coefficient, conductivity and specific heat may not.
- **US customary units.** GPM, lb/gal and bbl use US gallons (a barrel is 42 US gal).
- **Ranges.** Fractions above 1 (100 wt%, 10⁶ ppm, 100 mol%) are rejected.
  Temperature readings cannot go below absolute zero. Temperature differences
  and gauge pressures may be negative. Flow, composition, density and viscosity
  may not.
- **Absolute zero.** Readings within 10⁻⁹ K of absolute zero, such as
  −459.67 °F, are treated as exactly 0 K, since the gap is floating-point
  rounding.
- **Precision.** Calculations use `f64`, and every conversion factor is built
  from its exact SI definition. By default, output shows up to 10 significant
  figures with trailing zeros removed. `--sig-figs n` shows exactly n,
  keeping trailing zeros because they are significant. Neither implies more
  measurement accuracy than your inputs have.

## Library usage

Add a Git or path dependency:

```toml
[dependencies]
cheme-unit-converter = { git = "https://github.com/TehilaFavourite/cheme-unit-converter" }
```

Then parse units and call `convert`:

```rust
use cheme_unit_converter::{ConversionContext, convert, parse_unit};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let context = ConversionContext::new().with_density(850.0);
    let value = convert(42.5, parse_unit("cP")?, parse_unit("cSt")?, context)?;
    println!("{value} cSt");
    Ok(())
}
```

`Unit` also implements `FromStr` and `Display`, so `"cP".parse::<Unit>()` and
`unit.to_string()` work. Missing context is reported as
`ConversionError::MissingContext { item, reason }`. `item` is a `ContextItem`,
so callers can prompt for exactly the missing value. Unit names that don't
parse return a `ParseUnitError`.

`Unit`, the unit enums, the error enums and `ConversionContext` are
`#[non_exhaustive]`, so new units and context fields can be added in minor
releases. Build a context with `ConversionContext::new()` and its `with_*`
methods, and include a wildcard arm when matching on these enums.

The per-quantity types (`Temperature`, `Pressure`, `FlowRate`, `Energy`,
`HeatTransferCoefficient`, `SpecificHeat`, and so on) are also public. See
`cargo doc --open`.

## Web interface

**Live:** <https://tehilafavourite.github.io/cheme-unit-converter/>

The same Rust code runs in the browser as WebAssembly. Conversions update as you type, context fields
(density, atmospheric pressure, reference conditions, ...) appear only when the library reports them
missing, and every conversion is a shareable link such as
`?v=30&from=psig&to=bar&atm=101.325kPa`. A parity test runs the CLI and the web build on the same inputs
and requires identical output.

### Architecture

```text
 web/ (TypeScript, no framework)            wasm/ (Rust, cdylib)            root crate (Rust, no deps)
┌──────────────────────────────┐  JSON   ┌───────────────────────────┐   ┌─────────────────────────┐
│ main.ts   events → state     │────────▶│ catalogue()               │──▶│ Unit::all, parse_unit   │
│ state.ts  state ⇄ URL (pure) │         │ convert(request)          │──▶│ convert, Pressure::new… │
│ ui/*.ts   render functions   │◀────────│ resolveUnit(text)         │──▶│ format_value            │
│ wasm.ts   typed wrapper      │ typed   │ errors: kind/field/missing│   │ ConversionError, …      │
└──────────────────────────────┘ errors  └───────────────────────────┘   └─────────────────────────┘
```

- **Rust is the single source of truth:** units, aliases, parsing, validation, context rules, messages
  and number formatting. TypeScript holds state and renders.
- **The boundary is three JSON functions** that never throw. Errors are typed, so a missing context item
  reveals exactly the field it names.
- **Design decisions** are recorded in [docs/adr/](docs/adr/), and the visual design in
  [docs/design.md](docs/design.md).

### Local development

Requirements: Node 22 or 24, the `wasm32-unknown-unknown` target, `wasm-bindgen-cli` at the version in
`Cargo.lock`, and `wasm-opt` from binaryen.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version "=0.2.129" --locked
brew install binaryen               # or a binaryen release from GitHub

cd web
npm ci
npm run build:wasm                  # Rust → web/src/generated/
npm run dev                         # http://localhost:5173
```

Run `npm run build:wasm` again after changing Rust code.

### Tests

```sh
cargo test -p cheme-unit-converter-wasm                       # binding crate + CLI parity
cargo test -p cheme-unit-converter-wasm \
  --target wasm32-unknown-unknown --test web                  # wasm-bindgen-test in Node
cd web
npm test                                                      # Vitest: state and URL
npm run lint && npm run format:check && npx tsc --noEmit
npx playwright install chromium && npm run test:e2e           # Playwright + axe
npm run build && ../scripts/size-report.sh                    # gzip size budget
```

## Project layout

```text
cheme-unit-converter/
├── .github/
│   ├── actions/setup-web/     # Pinned web toolchain, shared by CI and Pages
│   └── workflows/             # ci.yml (Rust, MSRV, web), pages.yml (deploy)
├── CHANGELOG.md
├── Cargo.toml
├── LICENSE
├── README.md
├── docs/
│   ├── adr/             # Architecture decision records
│   └── design.md        # Visual design of the web interface
├── scripts/
│   ├── build-wasm.sh    # cargo → wasm-bindgen → wasm-opt
│   └── size-report.sh   # gzip sizes against the budget
├── tests/
│   ├── cli.rs           # Runs the binary: output, errors, exit status
│   └── round_trip.rs    # Every unit pair converts there and back
├── wasm/                # cheme-unit-converter-wasm: JSON bindings, parity tests
├── web/                 # Vite + TypeScript interface, Vitest and Playwright tests
└── src/
    ├── lib.rs           # Public API, convert(), shared test helpers
    ├── main.rs          # Command-line parsing and help
    ├── format.rs        # Result formatting shared by the CLI and web
    ├── unit.rs          # Unit registry, labels, aliases, parse_unit()
    ├── error.rs         # ConversionError, Quantity, ContextItem
    ├── context.rs       # ConversionContext
    ├── constants.rs     # Exact unit definitions and physical constants
    ├── temperature.rs   # Readings and differences
    ├── pressure.rs      # Absolute, gauge, manometer units
    ├── flow.rs          # Mass, volumetric, molar, standard gas flow
    ├── composition.rs   # Fractions, concentrations, molality
    ├── density.rs       # Density, SG, °API
    ├── viscosity.rs     # Dynamic and kinematic viscosity
    ├── energy.rs        # Energy and power
    ├── heat_transfer.rs # U, k, heat flux, specific heat, specific enthalpy
    └── linear.rs        # Shared code for factor-only quantities
```

Each quantity module keeps its own unit tests.

## License

MIT. See [LICENSE](LICENSE).
