//! End-to-end tests that run the compiled command-line program.

use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_cheme-unit-converter"))
        .args(args)
        .output()
        .expect("the binary should run")
}

fn stdout(output: &Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be UTF-8")
}

fn stderr(output: &Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be UTF-8")
}

/// Asserts a successful conversion and returns its single output line.
fn converted(args: &[&str]) -> String {
    let output = run(args);
    assert!(
        output.status.success(),
        "{args:?} failed: {}",
        stderr(&output)
    );
    stdout(&output).trim_end().to_string()
}

#[test]
fn converts_and_prints_labels() {
    assert_eq!(converted(&["convert", "25", "C", "F"]), "25 °C = 77 °F");
    assert_eq!(converted(&["convert", "10", "dC", "dF"]), "10 Δ°C = 18 Δ°F");
    assert_eq!(
        converted(&["convert", "-459.67", "F", "K"]),
        "-459.67 °F = 0 K"
    );
}

#[test]
fn default_output_has_ten_significant_figures() {
    assert_eq!(
        converted(&["convert", "1", "BTU/h", "kW"]),
        "1 BTU/h = 0.0002930710702 kW"
    );
    assert_eq!(
        converted(&["convert", "1", "kWh", "BTU"]),
        "1 kWh = 3412.141633 BTU"
    );
}

#[test]
fn sig_figs_keeps_significant_trailing_zeros() {
    assert_eq!(
        converted(&["convert", "25", "C", "F", "--sig-figs", "4"]),
        "25 °C = 77.00 °F"
    );
}

#[test]
fn options_may_come_before_positional_arguments() {
    assert_eq!(
        converted(&["convert", "--density", "1000", "1", "m3/s", "kg/h"]),
        "1 m³/s = 3600000 kg/h"
    );
}

#[test]
fn accepts_units_with_parentheses_and_punctuation() {
    assert_eq!(
        converted(&[
            "convert",
            "1",
            "BTU/(h·ft²·°F)",
            "w/m^2.k",
            "--sig-figs",
            "5"
        ]),
        "1 BTU/(h·ft²·°F) = 5.6783 W/(m²·K)"
    );
}

#[test]
fn standard_gas_flow_requires_every_condition() {
    let line = converted(&[
        "convert",
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
        "1",
        "--sig-figs",
        "5",
    ]);
    assert_eq!(line, "1000 Sm³/h = 261.04 m³/h");
}

#[test]
fn missing_context_exits_1_with_a_hint() {
    let output = run(&["convert", "30", "psig", "bar"]);
    assert_eq!(output.status.code(), Some(1));
    let message = stderr(&output);
    assert!(message.contains("requires the local atmospheric pressure"));
    assert!(message.contains("Hint: supply --atm-pressure"));
}

#[test]
fn invalid_usage_exits_2_with_short_usage() {
    for args in [
        &["convert", "1", "kg/h"][..],
        &["convert", "abc", "C", "F"],
        &["convert", "1", "C", "F", "--density"],
        &["convert", "1", "C", "F", "--bogus", "1"],
        &["frobnicate"],
        &[],
    ] {
        let output = run(args);
        assert_eq!(output.status.code(), Some(2), "{args:?}");
        let message = stderr(&output);
        assert!(message.starts_with("Error: "), "{args:?}: {message}");
        assert!(message.contains("--help"), "{args:?}: {message}");
        assert!(
            message.lines().count() < 12,
            "usage errors should not print the full help"
        );
    }
}

#[test]
fn unknown_unit_points_to_the_units_command() {
    let output = run(&["convert", "1", "furlong", "m"]);
    assert_eq!(output.status.code(), Some(2));
    assert!(stderr(&output).contains("cheme-unit-converter units"));
}

#[test]
fn incompatible_quantities_exit_1() {
    let output = run(&["convert", "1", "kWh", "kW"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(stderr(&output).contains("energy per unit time"));
}

#[test]
fn help_version_and_units_succeed() {
    let help = run(&["--help"]);
    assert!(help.status.success());
    assert!(stdout(&help).contains("Options"));

    let version = run(&["--version"]);
    assert!(version.status.success());
    assert_eq!(
        stdout(&version).trim(),
        format!("cheme-unit-converter {}", env!("CARGO_PKG_VERSION"))
    );

    let units = run(&["units"]);
    assert!(units.status.success());
    assert!(stdout(&units).contains("BTU/(h·ft²·°F)"));
}
