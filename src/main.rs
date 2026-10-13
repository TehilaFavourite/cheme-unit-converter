//! Command-line interface for the chemical engineering unit converter.
//!
//! Run `cheme-unit-converter --help` for options, or `cheme-unit-converter
//! units` for every accepted unit spelling. Examples:
//!
//! ```text
//! cheme-unit-converter convert 25 C F
//! cheme-unit-converter convert 10 dC dF
//! cheme-unit-converter convert 1 m3/s kg/h --density 1000
//! cheme-unit-converter convert 30 psig bar --atm-pressure 101.325 kPa
//! ```
//!
//! Exit status is 0 on success, 1 when a conversion fails, and 2 when the
//! command line itself is invalid.

use std::process::ExitCode;

use cheme_unit_converter::{
    ContextItem, ConversionContext, ConversionError, ParseUnitError, Pressure, PressureUnit,
    Quantity, Temperature, TemperatureUnit, Unit, convert, parse_unit,
};

const NAME: &str = "cheme-unit-converter";

const USAGE: &str = "\
Usage:
  cheme-unit-converter convert <value> <from-unit> <to-unit> [options]
  cheme-unit-converter units
  cheme-unit-converter --help | --version";

const OPTIONS: &str = "\
Options (supply only what a conversion needs; nothing is assumed):
  --density <kg/m3>              Fluid, solution, or mixture density
  --molar-mass <g/mol>           Molar mass of the flowing substance or solute
  --solvent-molar-mass <g/mol>   Molar mass of the solvent (rest of the mixture)
  --ref-density <kg/m3>          Reference density for SG and °API
  --atm-pressure <value> <unit>  Local atmospheric pressure, for gauge units
  --ref-temp <value> <unit>      Temperature that defines standard gas volume
  --ref-pressure <value> <unit>  Pressure that defines standard gas volume
  --temp <value> <unit>          Actual gas temperature
  --pressure <value> <unit>      Actual gas pressure
  --z <value>                    Gas compressibility factor at actual conditions
                                 (1 for an ideal gas)
  --sig-figs <n>                 Round the result to n significant figures
                                 (default: up to 10, trailing zeros trimmed)

Examples:
  cheme-unit-converter convert 25 C F
  cheme-unit-converter convert 10 dC dF
  cheme-unit-converter convert 30 psig bar --atm-pressure 101.325 kPa
  cheme-unit-converter convert 100 kmol/h Sm3/h --ref-temp 0 C --ref-pressure 1 atm
  cheme-unit-converter convert 1.2 mol/L wt% --density 1045 --molar-mass 58.44
  cheme-unit-converter convert 150 \"BTU/(h·ft2·F)\" W/m2K
  cheme-unit-converter convert 40.65 kJ/mol kJ/kg --molar-mass 18.015

Exit status: 0 on success, 1 if the conversion fails, 2 for invalid usage.";

const DEFAULT_SIGNIFICANT_FIGURES: usize = 10;
const MAX_SIGNIFICANT_FIGURES: usize = 17;

/// A command failure, split by whether the command line or the conversion
/// was at fault so each can get its own exit status.
#[derive(Debug)]
enum CliError {
    /// The arguments are malformed. Exit status 2.
    Usage(String),
    /// The arguments are well formed but the conversion is not possible.
    /// Exit status 1.
    Conversion(String),
}

impl From<ParseUnitError> for CliError {
    fn from(error: ParseUnitError) -> Self {
        let message = match error {
            ParseUnitError::Unrecognized(_) => {
                format!("{error}\nRun '{NAME} units' to list supported units.")
            }
            _ => error.to_string(),
        };
        Self::Usage(message)
    }
}

impl From<ConversionError> for CliError {
    fn from(error: ConversionError) -> Self {
        Self::Conversion(explain(error))
    }
}

fn usage_error(message: impl Into<String>) -> CliError {
    CliError::Usage(message.into())
}

/// Runs one command and maps failures to exit statuses.
fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(CliError::Usage(message)) => {
            eprintln!("Error: {message}\n\n{USAGE}\n\nRun '{NAME} --help' for units and options.");
            ExitCode::from(2)
        }
        Err(CliError::Conversion(message)) => {
            eprintln!("Error: {message}");
            ExitCode::from(1)
        }
    }
}

/// Dispatches to the requested command.
fn run(args: impl IntoIterator<Item = String>) -> Result<(), CliError> {
    let args: Vec<String> = args.into_iter().collect();
    let Some(command) = args.first() else {
        return Err(usage_error("missing command"));
    };
    let rest = &args[1..];
    match command.as_str() {
        "-h" | "--help" | "help" => println!("{}", help()),
        "-V" | "--version" => println!("{NAME} {}", env!("CARGO_PKG_VERSION")),
        "units" if rest.is_empty() => print!("{}", unit_listing()),
        "units" => return Err(usage_error("'units' takes no arguments")),
        "convert" => convert_command(rest)?,
        other => return Err(usage_error(format!("unrecognized command '{other}'"))),
    }
    Ok(())
}

/// Performs one conversion and prints the formatted result.
fn convert_command(args: &[String]) -> Result<(), CliError> {
    let (positionals, options) = split_arguments(args)?;
    let [input, from, to] = positionals.as_slice() else {
        return Err(usage_error(format!(
            "'convert' expects <value> <from-unit> <to-unit>, but got {} argument(s)",
            positionals.len()
        )));
    };
    let input = input.trim();
    let value = input
        .parse::<f64>()
        .map_err(|_| usage_error(format!("invalid numeric value '{input}'")))?;
    let from = parse_unit(from)?;
    let to = parse_unit(to)?;
    let options = parse_options(&options)?;
    let result = convert(value, from, to, options.context)?;

    let digits = options
        .significant_figures
        .unwrap_or(DEFAULT_SIGNIFICANT_FIGURES);
    let formatted = format_significant(result, digits);
    let formatted = if options.significant_figures.is_some() {
        formatted
    } else {
        trim_insignificant_zeros(&formatted)
    };
    println!("{input} {from} = {formatted} {to}");
    Ok(())
}

/// An option name with the values that follow it.
type RawOption<'a> = (&'a str, Vec<&'a str>);

/// Returns how many values follow an option, or `None` if it is unknown.
fn option_arity(option: &str) -> Option<usize> {
    match option {
        "--density"
        | "--molar-mass"
        | "--solvent-molar-mass"
        | "--ref-density"
        | "--z"
        | "--sig-figs" => Some(1),
        "--atm-pressure" | "--ref-pressure" | "--pressure" | "--ref-temp" | "--temp" => Some(2),
        _ => None,
    }
}

/// Separates positional arguments from options, so options may appear
/// anywhere after `convert`.
fn split_arguments(args: &[String]) -> Result<(Vec<&str>, Vec<RawOption<'_>>), CliError> {
    let mut positionals = Vec::new();
    let mut options = Vec::new();
    let mut args = args.iter().map(String::as_str);
    while let Some(arg) = args.next() {
        if !arg.starts_with("--") {
            positionals.push(arg);
            continue;
        }
        let arity =
            option_arity(arg).ok_or_else(|| usage_error(format!("unrecognized option '{arg}'")))?;
        let values: Vec<&str> = args.by_ref().take(arity).collect();
        if values.len() < arity || values.iter().any(|value| value.starts_with("--")) {
            let expected = if arity == 1 {
                "a value"
            } else {
                "a value and a unit, for example '1 atm'"
            };
            return Err(usage_error(format!("option '{arg}' needs {expected}")));
        }
        options.push((arg, values));
    }
    Ok((positionals, options))
}

#[derive(Debug, Default)]
struct Options {
    context: ConversionContext,
    significant_figures: Option<usize>,
}

/// Options as given on the command line, before pressures and temperatures
/// are validated. Pressures are built last so gauge values can use the
/// atmospheric pressure regardless of option order.
#[derive(Default)]
struct RawOptions {
    density: Option<f64>,
    molar_mass: Option<f64>,
    solvent_molar_mass: Option<f64>,
    reference_density: Option<f64>,
    compressibility_factor: Option<f64>,
    atmospheric_pressure: Option<(f64, PressureUnit)>,
    reference_pressure: Option<(f64, PressureUnit)>,
    pressure: Option<(f64, PressureUnit)>,
    reference_temperature: Option<(f64, TemperatureUnit)>,
    temperature: Option<(f64, TemperatureUnit)>,
    significant_figures: Option<usize>,
}

/// Reads physical properties, reference conditions, and output settings.
fn parse_options(options: &[RawOption<'_>]) -> Result<Options, CliError> {
    let mut raw = RawOptions::default();
    for (option, values) in options {
        let option = *option;
        match option {
            "--density" => set_once(&mut raw.density, number(option, values[0])?, option)?,
            "--molar-mass" => set_once(&mut raw.molar_mass, number(option, values[0])?, option)?,
            "--solvent-molar-mass" => set_once(
                &mut raw.solvent_molar_mass,
                number(option, values[0])?,
                option,
            )?,
            "--ref-density" => set_once(
                &mut raw.reference_density,
                number(option, values[0])?,
                option,
            )?,
            "--z" => set_once(
                &mut raw.compressibility_factor,
                number(option, values[0])?,
                option,
            )?,
            "--atm-pressure" => {
                let value = pressure_option(option, values)?;
                if value.1.is_gauge() {
                    return Err(usage_error(
                        "option '--atm-pressure' must use an absolute pressure unit",
                    ));
                }
                set_once(&mut raw.atmospheric_pressure, value, option)?;
            }
            "--ref-pressure" => set_once(
                &mut raw.reference_pressure,
                pressure_option(option, values)?,
                option,
            )?,
            "--pressure" => set_once(&mut raw.pressure, pressure_option(option, values)?, option)?,
            "--ref-temp" => set_once(
                &mut raw.reference_temperature,
                temperature_option(option, values)?,
                option,
            )?,
            "--temp" => set_once(
                &mut raw.temperature,
                temperature_option(option, values)?,
                option,
            )?,
            "--sig-figs" => {
                let digits = values[0]
                    .parse::<usize>()
                    .ok()
                    .filter(|digits| (1..=MAX_SIGNIFICANT_FIGURES).contains(digits))
                    .ok_or_else(|| {
                        usage_error(format!(
                            "option '--sig-figs' expects a whole number from 1 to \
                             {MAX_SIGNIFICANT_FIGURES}"
                        ))
                    })?;
                set_once(&mut raw.significant_figures, digits, option)?;
            }
            _ => unreachable!("split_arguments only accepts known options"),
        }
    }

    let mut context = ConversionContext::new();
    context.density_kg_m3 = raw.density;
    context.molar_mass_g_mol = raw.molar_mass;
    context.solvent_molar_mass_g_mol = raw.solvent_molar_mass;
    context.reference_density_kg_m3 = raw.reference_density;
    context.compressibility_factor = raw.compressibility_factor;
    context.reference_temperature = build_temperature("--ref-temp", raw.reference_temperature)?;
    context.temperature = build_temperature("--temp", raw.temperature)?;
    context.atmospheric_pressure =
        build_pressure("--atm-pressure", raw.atmospheric_pressure, &context)?;
    context.reference_pressure =
        build_pressure("--ref-pressure", raw.reference_pressure, &context)?;
    context.pressure = build_pressure("--pressure", raw.pressure, &context)?;

    Ok(Options {
        context,
        significant_figures: raw.significant_figures,
    })
}

fn set_once<T>(slot: &mut Option<T>, value: T, option: &str) -> Result<(), CliError> {
    if slot.replace(value).is_some() {
        return Err(usage_error(format!(
            "option '{option}' may only be specified once"
        )));
    }
    Ok(())
}

fn number(option: &str, value: &str) -> Result<f64, CliError> {
    value.parse::<f64>().map_err(|_| {
        usage_error(format!(
            "invalid numeric value '{value}' for option '{option}'"
        ))
    })
}

fn pressure_option(option: &str, values: &[&str]) -> Result<(f64, PressureUnit), CliError> {
    match (number(option, values[0])?, parse_unit(values[1])?) {
        (value, Unit::Pressure(unit)) => Ok((value, unit)),
        _ => Err(usage_error(format!(
            "option '{option}' expects a pressure unit such as kPa, bar, or psi"
        ))),
    }
}

fn temperature_option(option: &str, values: &[&str]) -> Result<(f64, TemperatureUnit), CliError> {
    match (number(option, values[0])?, parse_unit(values[1])?) {
        (value, Unit::Temperature(unit)) => Ok((value, unit)),
        _ => Err(usage_error(format!(
            "option '{option}' expects a temperature unit such as C, F, K, or R"
        ))),
    }
}

fn build_pressure(
    option: &str,
    raw: Option<(f64, PressureUnit)>,
    context: &ConversionContext,
) -> Result<Option<Pressure>, CliError> {
    raw.map(|(value, unit)| Pressure::new(value, unit, context))
        .transpose()
        .map_err(|error| CliError::Conversion(format!("{option}: {}", explain(error))))
}

fn build_temperature(
    option: &str,
    raw: Option<(f64, TemperatureUnit)>,
) -> Result<Option<Temperature>, CliError> {
    raw.map(|(value, unit)| Temperature::new(value, unit))
        .transpose()
        .map_err(|error| CliError::Conversion(format!("{option}: {error}")))
}

/// Adds the command-line option that supplies missing or invalid context.
fn explain(error: ConversionError) -> String {
    let item = match &error {
        ConversionError::MissingContext { item, .. } | ConversionError::InvalidContext(item) => {
            *item
        }
        _ => return error.to_string(),
    };
    let hint = match item {
        ContextItem::Density => "--density <kg/m3>",
        ContextItem::MolarMass => "--molar-mass <g/mol>",
        ContextItem::SolventMolarMass => "--solvent-molar-mass <g/mol>",
        ContextItem::ReferenceDensity => {
            "--ref-density <kg/m3>, for example the density of water at the reference temperature"
        }
        ContextItem::AtmosphericPressure => {
            "--atm-pressure <value> <unit>, for example --atm-pressure 101.325 kPa"
        }
        ContextItem::ReferenceTemperature => {
            "--ref-temp <value> <unit>; check which standard applies (0 C, 15 C, 20 C, and 60 F \
             are all in use)"
        }
        ContextItem::ReferencePressure => {
            "--ref-pressure <value> <unit>; check which standard applies (1 atm and 1 bar are \
             both in use)"
        }
        ContextItem::OperatingTemperature => "--temp <value> <unit>",
        ContextItem::OperatingPressure => {
            "--pressure <value> <unit> (absolute, or gauge together with --atm-pressure)"
        }
        ContextItem::CompressibilityFactor => "--z <value>; use 1 to treat the gas as ideal",
        _ => return error.to_string(),
    };
    format!("{error}\nHint: supply {hint}")
}

/// Groups units by quantity, preserving registry order.
fn units_by_quantity() -> Vec<(Quantity, Vec<Unit>)> {
    let mut groups: Vec<(Quantity, Vec<Unit>)> = Vec::new();
    for unit in Unit::all() {
        match groups.last_mut() {
            Some((quantity, units)) if *quantity == unit.quantity() => units.push(unit),
            _ => groups.push((unit.quantity(), vec![unit])),
        }
    }
    groups
}

fn heading(quantity: Quantity) -> String {
    let name = quantity.name();
    let mut chars = name.chars();
    chars
        .next()
        .map(|first| first.to_uppercase().chain(chars).collect())
        .unwrap_or_default()
}

fn help() -> String {
    const LABEL_WIDTH: usize = 28;
    const LINE_WIDTH: usize = 80;
    let mut text = format!(
        "Chemical Engineering Unit Converter {}\n\n{USAGE}\n\n\
         Units (case- and punctuation-insensitive; 'units' lists extra spellings):\n",
        env!("CARGO_PKG_VERSION")
    );
    for (quantity, units) in units_by_quantity() {
        let mut line = format!("  {:<LABEL_WIDTH$}", format!("{}:", heading(quantity)));
        let mut line_length = line.chars().count();
        for (index, unit) in units.iter().enumerate() {
            let item = if index + 1 < units.len() {
                format!("{unit}, ")
            } else {
                unit.to_string()
            };
            if line_length + item.chars().count() > LINE_WIDTH {
                text += line.trim_end();
                text.push('\n');
                line = " ".repeat(LABEL_WIDTH + 2);
                line_length = line.len();
            }
            line_length += item.chars().count();
            line += &item;
        }
        text += &line;
        text.push('\n');
    }
    text + "\n" + OPTIONS
}

fn unit_listing() -> String {
    let mut text = String::from(
        "Matching ignores case, spaces, ·, *, ., -, ^, °, and parentheses; ² and ³ may be\n\
         typed as 2 and 3, and hr as h. So C matches °C, and W/m2K matches W/(m²·K).\n\
         Each unit is followed by any additional spellings it accepts.\n\n",
    );
    for (quantity, units) in units_by_quantity() {
        text += &heading(quantity);
        text.push('\n');
        for unit in units {
            let line = format!("  {:<16}{}", unit.label(), unit.aliases().join(", "));
            text += line.trim_end();
            text.push('\n');
        }
        text.push('\n');
    }
    text
}

/// Removes trailing zeros after the decimal point, in fixed or scientific
/// notation. Used for default output, where trailing zeros carry no meaning.
fn trim_insignificant_zeros(number: &str) -> String {
    fn trim_fraction(number: &str) -> &str {
        if number.contains('.') {
            number.trim_end_matches('0').trim_end_matches('.')
        } else {
            number
        }
    }
    match number.split_once('e') {
        Some((mantissa, exponent)) => format!("{}e{exponent}", trim_fraction(mantissa)),
        None => trim_fraction(number).to_string(),
    }
}

/// Formats `value` to `digits` significant figures, keeping trailing zeros
/// because they are significant. Uses scientific notation, as `%g` does,
/// when the exponent is below −4 or at least `digits`.
fn format_significant(value: f64, digits: usize) -> String {
    // Adding 0.0 turns −0 into +0, so a zero result never prints as "-0".
    let value = value + 0.0;
    let scientific = format!("{value:.precision$e}", precision = digits - 1);
    let (_, exponent) = scientific
        .split_once('e')
        .expect("scientific formatting always has an exponent");
    let exponent: i32 = exponent.parse().expect("exponent is an integer");
    let digits = i32::try_from(digits).expect("at most MAX_SIGNIFICANT_FIGURES digits");
    if (-4..digits).contains(&exponent) {
        let decimals = usize::try_from(digits - 1 - exponent).unwrap_or(0);
        format!("{value:.decimals$}")
    } else {
        scientific
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| value.to_string()).collect()
    }

    fn options(values: &[&str]) -> Result<Options, CliError> {
        let args = strings(values);
        let (_, raw) = split_arguments(&args)?;
        parse_options(&raw)
    }

    #[test]
    fn formats_significant_figures() {
        assert_eq!(format_significant(77.0, 3), "77.0");
        assert_eq!(format_significant(9.996, 3), "10.0");
        assert_eq!(format_significant(12_345.0, 3), "1.23e4");
        assert_eq!(format_significant(0.000_123_4, 2), "0.00012");
        assert_eq!(format_significant(0.000_012_34, 2), "1.2e-5");
        assert_eq!(format_significant(0.0, 3), "0.00");
        assert_eq!(format_significant(-0.0, 3), "0.00");
    }

    #[test]
    fn default_format_keeps_ten_significant_figures() {
        let default = |value| trim_insignificant_zeros(&format_significant(value, 10));
        assert_eq!(default(212.0), "212");
        assert_eq!(default(0.25), "0.25");
        assert_eq!(default(1e-12), "1e-12");
        assert_eq!(default(0.000_293_071_070_172_222), "0.0002930710702");
        assert_eq!(default(-0.0), "0");
    }

    #[test]
    fn options_may_appear_anywhere() {
        let args = strings(&["--density", "1000", "1", "m3/s", "kg/h"]);
        let (positionals, raw) = split_arguments(&args).unwrap();
        assert_eq!(positionals, ["1", "m3/s", "kg/h"]);
        assert_eq!(raw, [("--density", vec!["1000"])]);
    }

    #[test]
    fn builds_gauge_pressure_options_in_any_order() {
        let parsed =
            options(&["--pressure", "50", "psig", "--atm-pressure", "14.7", "psi"]).unwrap();
        let pressure = parsed.context.pressure.unwrap();
        let psi = Pressure::new(64.7, PressureUnit::Psi, &ConversionContext::default()).unwrap();
        assert!((pressure.pascals() - psi.pascals()).abs() < 1e-6);
    }

    #[test]
    fn rejects_bad_options() {
        assert!(options(&["--density", "1000", "--density", "998"]).is_err());
        assert!(options(&["--ref-temp", "0"]).is_err());
        assert!(options(&["--ref-temp", "0", "dC"]).is_err());
        assert!(options(&["--atm-pressure", "0", "psig"]).is_err());
        assert!(options(&["--sig-figs", "0"]).is_err());
        assert!(options(&["--density", "--z", "1"]).is_err());
        assert!(options(&["--unknown", "1"]).is_err());
    }

    #[test]
    fn missing_context_errors_name_the_option() {
        let error = convert(
            1.0,
            parse_unit("psig").unwrap(),
            parse_unit("psi").unwrap(),
            ConversionContext::default(),
        )
        .unwrap_err();
        assert!(explain(error).contains("--atm-pressure"));
    }
}
