//! Asserts that the web interface and the command-line interface give the
//! same output for the same input, so the two can never drift apart.
//!
//! Each case is a CLI argument list. The test runs the real CLI binary on it,
//! sends the equivalent request to the binding crate, and compares the CLI's
//! output line (or error message) with the one built from the response.
#![cfg(not(target_arch = "wasm32"))]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use cheme_unit_converter_wasm::api::{ConvertRequest, ConvertResponse, convert};
use cheme_unit_converter_wasm::context::QuantityInput;
use serde_json::Value;

/// Arguments after `convert`, exactly as typed on the command line.
const SUCCESSES: &[&[&str]] = &[
    // Temperature and temperature difference
    &["25", "C", "F"],
    &["-40", "F", "C"],
    &["0", "R", "K", "--sig-figs", "3"],
    &["-459.67", "F", "K"],
    &["10", "dC", "dF"],
    &["1.8", "dR", "dK"],
    // Pressure, including gauge with stated atmosphere
    &["1", "atm", "kPa"],
    &["30", "psig", "bar", "--atm-pressure", "101.325", "kPa"],
    &["760", "torr", "inHg", "--sig-figs", "6"],
    &["1", "barg", "psig"],
    // Flow: within and across bases, standard gas, actual gas
    &["3600", "kg/h", "lb/min"],
    &["1", "GPM", "L/min"],
    &["1", "m3/s", "kg/h", "--density", "1000"],
    &[
        "100",
        "kmol/h",
        "Sm3/h",
        "--ref-temp",
        "0",
        "C",
        "--ref-pressure",
        "1",
        "atm",
    ],
    &[
        "500",
        "SCFM",
        "kg/h",
        "--ref-temp",
        "60",
        "F",
        "--ref-pressure",
        "14.696",
        "psi",
        "--molar-mass",
        "28.97",
    ],
    &[
        "1000",
        "Sm3/h",
        "m3/h",
        "--ref-temp",
        "0",
        "C",
        "--ref-pressure",
        "1",
        "atm",
        "--temp",
        "150",
        "C",
        "--pressure",
        "5",
        "barg",
        "--atm-pressure",
        "1.01325",
        "bar",
        "--z",
        "0.95",
    ],
    // Composition
    &["0.25", "wt%", "ppm"],
    &[
        "1.2",
        "mol/L",
        "wt%",
        "--density",
        "1045",
        "--molar-mass",
        "58.44",
        "--sig-figs",
        "3",
    ],
    &[
        "0.3",
        "x",
        "wt%",
        "--molar-mass",
        "46.07",
        "--solvent-molar-mass",
        "18.015",
    ],
    &["50", "wt%", "mol/kg", "--molar-mass", "100"],
    // Density
    &["0.85", "SG", "API"],
    &["0.85", "SG", "kg/m3", "--ref-density", "999"],
    &["1", "g/cm3", "lb/ft3"],
    // Viscosity
    &["42.5", "cP", "cSt", "--density", "850"],
    &["1", "P", "mPa.s"],
    // Energy and power
    &["1", "kWh", "BTU"],
    &["1", "kcal_IT", "kJ"],
    &["500", "TR", "kW"],
    &["1", "hp", "kW", "--sig-figs", "5"],
    &["1", "kcal/h", "W"],
    // Heat transfer
    &["150", "BTU/(h·ft2·F)", "W/m2K"],
    &["0.6", "W/mK", "BTU/hr.ft.F"],
    &["10000", "W/m2", "BTU/(h·ft²)"],
    &["1", "BTU/(lb·°F)", "kJ/(kg·K)"],
    &["75.3", "J/molK", "kJ/kgK", "--molar-mass", "18.015"],
    &["40.65", "kJ/mol", "kJ/kg", "--molar-mass", "18.015"],
    &["-100", "kJ/kg", "BTU/lb"],
    // Formatting extremes
    &["1e-12", "J", "kJ"],
    &["1e20", "J", "GJ"],
    &["12345", "J", "J", "--sig-figs", "3"],
];

/// Cases the CLI rejects; the Wasm error message must match the CLI's.
const FAILURES: &[&[&str]] = &[
    &["30", "psig", "bar"],
    &["1", "cP", "cSt"],
    &["0.3", "x", "wt%", "--molar-mass", "46.07"],
    &["1", "cP", "cSt", "--density", "0"],
    &["1", "kWh", "kW"],
    &["1", "C", "dF"],
    &["-1", "kg/h", "kg/s"],
    &["-500", "C", "K"],
    &["101", "wt%", "ppm"],
    &["1e300", "GJ", "J"],
    &["1", "furlong", "m"],
    &["1", "C", "mPa"],
    &["abc", "C", "F"],
];

/// Builds the CLI once, in its own target directory so it does not contend
/// with the `cargo test` that is running this test.
fn cli() -> &'static Path {
    static BINARY: OnceLock<Result<PathBuf, String>> = OnceLock::new();
    match BINARY.get_or_init(build_cli) {
        Ok(path) => path,
        Err(error) => panic!("could not build the CLI: {error}"),
    }
}

fn build_cli() -> Result<PathBuf, String> {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or("the wasm crate has no parent directory")?;
    let output = Command::new(env!("CARGO"))
        .current_dir(workspace)
        .args(["build", "--quiet", "--message-format=json"])
        .args([
            "--package",
            "cheme-unit-converter",
            "--bin",
            "cheme-unit-converter",
        ])
        .arg("--target-dir")
        .arg(Path::new(env!("CARGO_TARGET_TMPDIR")).join("parity-cli"))
        .output()
        .map_err(|error| error.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).into_owned());
    }
    String::from_utf8_lossy(&output.stdout)
        .lines()
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .find(|message| {
            message["reason"] == "compiler-artifact"
                && message["target"]["name"] == "cheme-unit-converter"
                && message["executable"].is_string()
        })
        .and_then(|message| message["executable"].as_str().map(PathBuf::from))
        .ok_or_else(|| "cargo reported no CLI executable".to_string())
}

/// What the CLI printed: the result line, or the first line of the error
/// without its "Error: " prefix (later lines are CLI-only hints).
fn run_cli(args: &[&str]) -> Result<String, String> {
    let output = Command::new(cli())
        .arg("convert")
        .args(args)
        .output()
        .map_err(|error| format!("could not run the CLI: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout)
        .trim_end()
        .to_string();
    let stderr = String::from_utf8_lossy(&output.stderr);
    let first_line = stderr.lines().next().unwrap_or_default();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(first_line.trim_start_matches("Error: ").to_string())
    }
}

/// Translates CLI arguments into the equivalent web request.
fn request(args: &[&str]) -> ConvertRequest {
    let [value, from, to, options @ ..] = args else {
        panic!("a case needs a value and two units: {args:?}");
    };
    let mut request = ConvertRequest {
        value: (*value).into(),
        from: (*from).into(),
        to: (*to).into(),
        ..ConvertRequest::default()
    };
    let context = &mut request.context;
    let mut options = options.iter();
    while let Some(&option) = options.next() {
        let mut next = || {
            options
                .next()
                .map(|text| text.to_string())
                .unwrap_or_default()
        };
        let mut quantity = || QuantityInput {
            value: next(),
            unit: next(),
        };
        match option {
            "--density" => context.density = Some(next()),
            "--molar-mass" => context.molar_mass = Some(next()),
            "--solvent-molar-mass" => context.solvent_molar_mass = Some(next()),
            "--ref-density" => context.reference_density = Some(next()),
            "--z" => context.compressibility_factor = Some(next()),
            "--atm-pressure" => context.atmospheric_pressure = Some(quantity()),
            "--ref-temp" => context.reference_temperature = Some(quantity()),
            "--ref-pressure" => context.reference_pressure = Some(quantity()),
            "--temp" => context.temperature = Some(quantity()),
            "--pressure" => context.pressure = Some(quantity()),
            "--sig-figs" => request.sig_figs = Some(next()),
            other => panic!("the parity test does not know option {other}"),
        }
    }
    request
}

/// What the web interface shows for the same input, in the CLI's layout.
fn run_wasm(args: &[&str]) -> (Result<String, String>, Option<&'static str>) {
    let request = request(args);
    match convert(&request) {
        ConvertResponse::Success(success) => (
            Ok(format!(
                "{} {} = {} {}",
                request.value.trim(),
                success.from,
                success.formatted,
                success.to
            )),
            Some(success.quantity),
        ),
        ConvertResponse::Failure(failure) => (Err(failure.error.message), None),
    }
}

#[test]
fn web_and_cli_agree_on_successful_conversions() {
    let mut quantities = BTreeSet::new();
    let mismatches: Vec<String> = SUCCESSES
        .iter()
        .filter_map(|args| {
            let cli = run_cli(args);
            let (web, quantity) = run_wasm(args);
            quantities.extend(quantity);
            (cli.is_err() || cli != web)
                .then(|| format!("{args:?}\n  cli: {cli:?}\n  web: {web:?}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
    assert!(SUCCESSES.len() >= 30, "keep at least 30 success cases");
    assert_eq!(quantities.len(), 14, "cover every quantity: {quantities:?}");
}

#[test]
fn web_and_cli_agree_on_error_messages() {
    let mismatches: Vec<String> = FAILURES
        .iter()
        .filter_map(|args| {
            let cli = run_cli(args);
            let (web, _) = run_wasm(args);
            (cli.is_ok() || cli != web).then(|| format!("{args:?}\n  cli: {cli:?}\n  web: {web:?}"))
        })
        .collect();
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}
