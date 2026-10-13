//! Request handlers behind the WebAssembly exports, as plain Rust so they can
//! be tested natively. See `docs/adr/0003-wasm-boundary-design.md` for the
//! contract.

use cheme_unit_converter::{
    ConversionError, DEFAULT_SIGNIFICANT_FIGURES, MAX_SIGNIFICANT_FIGURES, ParseUnitError,
    Quantity, SignificantFigures, Unit, format_value, parse_unit,
};
use serde::{Deserialize, Serialize};

use crate::context::{ContextInputs, ContextKey, ContextRequest, parse_number};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// What went wrong, as a stable discriminant the interface can branch on.
pub enum ErrorKind {
    /// A required input is blank.
    EmptyInput,
    InvalidNumber,
    UnrecognizedUnit,
    AmbiguousPrefix,
    InvalidSigFigs,
    /// A context field was given a unit of the wrong quantity.
    WrongContextUnit,
    IncompatibleUnits,
    MissingContext,
    InvalidContext,
    OutOfRange,
    NegativeValue,
    NegativePressure,
    BelowAbsoluteZero,
    NonFiniteValue,
    ResultOverflow,
    /// The request could not be read, or the library returned something this
    /// crate does not know about. Indicates a bug, not a user error.
    Internal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// The main inputs of the form.
pub enum InputField {
    Value,
    From,
    To,
    SigFigs,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(untagged)]
/// The input responsible for an error: a main input or a context field.
pub enum Field {
    Input(InputField),
    Context(ContextKey),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// A typed error. `message` is the library's own text where it has one.
pub struct ApiError {
    pub kind: ErrorKind,
    pub message: String,
    pub field: Field,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub missing: Option<ContextKey>,
}

impl ApiError {
    pub fn new(kind: ErrorKind, field: Field, message: String) -> Self {
        Self {
            kind,
            message,
            field,
            missing: None,
        }
    }

    pub fn internal(message: String) -> Self {
        Self::new(
            ErrorKind::Internal,
            Field::Input(InputField::Value),
            message,
        )
    }

    /// Maps a unit-name error to the field it was typed in.
    pub fn unit(error: &ParseUnitError, field: Field) -> Self {
        let kind = match error {
            ParseUnitError::Unrecognized(_) => ErrorKind::UnrecognizedUnit,
            ParseUnitError::AmbiguousPrefix { .. } => ErrorKind::AmbiguousPrefix,
            _ => ErrorKind::Internal,
        };
        Self::new(kind, field, error.to_string())
    }

    /// Maps a library error to the field responsible.
    ///
    /// `field` is where the failing value came from: the main value for a
    /// conversion, or a context field while that field's own quantity is
    /// being built (a gauge pressure needing atmospheric pressure is reported
    /// against the gauge pressure).
    pub fn conversion(error: &ConversionError, field: Field) -> Self {
        let message = error.to_string();
        let unknown = || Self::internal(format!("unsupported library error: {message}"));
        let (kind, field, missing) = match error {
            ConversionError::MissingContext { item, .. } => {
                let Some(missing) = ContextKey::from_item(*item) else {
                    return unknown();
                };
                let field = match field {
                    Field::Context(_) => field,
                    Field::Input(_) => Field::Context(missing),
                };
                (ErrorKind::MissingContext, field, Some(missing))
            }
            ConversionError::InvalidContext(item) => {
                let Some(key) = ContextKey::from_item(*item) else {
                    return unknown();
                };
                (ErrorKind::InvalidContext, Field::Context(key), None)
            }
            ConversionError::IncompatibleUnits { .. } => (
                ErrorKind::IncompatibleUnits,
                Field::Input(InputField::To),
                None,
            ),
            ConversionError::NonFiniteValue => (ErrorKind::NonFiniteValue, field, None),
            ConversionError::ResultOverflow => (ErrorKind::ResultOverflow, field, None),
            ConversionError::BelowAbsoluteZero => (ErrorKind::BelowAbsoluteZero, field, None),
            ConversionError::NegativePressure => (ErrorKind::NegativePressure, field, None),
            ConversionError::NegativeValue(_) => (ErrorKind::NegativeValue, field, None),
            ConversionError::OutOfRange(_) => (ErrorKind::OutOfRange, field, None),
            _ => return unknown(),
        };
        Self {
            kind,
            message,
            field,
            missing,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
/// A conversion request. Numbers are text, parsed here exactly as the CLI
/// parses its arguments.
pub struct ConvertRequest {
    pub value: String,
    pub from: String,
    pub to: String,
    #[serde(default)]
    pub sig_figs: Option<String>,
    #[serde(default)]
    pub context: ContextRequest,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConvertSuccess {
    pub ok: bool,
    pub value: f64,
    /// The result formatted with the library's `format_value`.
    pub formatted: String,
    /// Display label of the source unit.
    pub from: &'static str,
    /// Display label of the target unit.
    pub to: &'static str,
    pub quantity: &'static str,
    /// Supplied context fields that this conversion depends on.
    pub used: Vec<ContextKey>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Failure {
    pub ok: bool,
    pub error: ApiError,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ConvertResponse {
    Success(ConvertSuccess),
    Failure(Failure),
}

impl ConvertResponse {
    pub fn failure(error: ApiError) -> Self {
        Self::Failure(Failure { ok: false, error })
    }
}

/// Converts a request, never failing: every error is part of the response.
pub fn convert(request: &ConvertRequest) -> ConvertResponse {
    match try_convert(request) {
        Ok(success) => ConvertResponse::Success(success),
        Err(error) => ConvertResponse::failure(error),
    }
}

fn try_convert(request: &ConvertRequest) -> Result<ConvertSuccess, ApiError> {
    let value = required(&request.value, InputField::Value, "enter a value")?;
    let value = parse_number(value, Field::Input(InputField::Value))?;
    let from = unit(
        &request.from,
        InputField::From,
        "choose the unit to convert from",
    )?;
    let to = unit(&request.to, InputField::To, "choose the unit to convert to")?;
    let significant_figures = significant_figures(request.sig_figs.as_deref())?;
    let inputs = ContextInputs::parse(&request.context)?;
    let result = run(value, from, to, &inputs)?;
    // A supplied item is used exactly when removing it makes the conversion
    // fail, because the library never falls back to a default value.
    let used = inputs
        .supplied()
        .into_iter()
        .filter(|&key| run(value, from, to, &inputs.without(key)).is_err())
        .collect();
    Ok(ConvertSuccess {
        ok: true,
        value: result,
        formatted: format_value(result, significant_figures),
        from: from.label(),
        to: to.label(),
        quantity: quantity_key(from.quantity())?,
        used,
    })
}

fn run(value: f64, from: Unit, to: Unit, inputs: &ContextInputs) -> Result<f64, ApiError> {
    let context = inputs.build()?;
    cheme_unit_converter::convert(value, from, to, context)
        .map_err(|error| ApiError::conversion(&error, Field::Input(InputField::Value)))
}

fn required<'a>(text: &'a str, field: InputField, message: &str) -> Result<&'a str, ApiError> {
    let text = text.trim();
    if text.is_empty() {
        Err(ApiError::new(
            ErrorKind::EmptyInput,
            Field::Input(field),
            message.to_string(),
        ))
    } else {
        Ok(text)
    }
}

fn unit(text: &str, field: InputField, empty_message: &str) -> Result<Unit, ApiError> {
    let text = required(text, field, empty_message)?;
    parse_unit(text).map_err(|error| ApiError::unit(&error, Field::Input(field)))
}

fn significant_figures(text: Option<&str>) -> Result<Option<SignificantFigures>, ApiError> {
    let Some(text) = text.map(str::trim).filter(|text| !text.is_empty()) else {
        return Ok(None);
    };
    text.parse::<usize>()
        .ok()
        .and_then(SignificantFigures::new)
        .map(Some)
        .ok_or_else(|| {
            ApiError::new(
                ErrorKind::InvalidSigFigs,
                Field::Input(InputField::SigFigs),
                format!(
                    "significant figures must be a whole number from 1 to {MAX_SIGNIFICANT_FIGURES}"
                ),
            )
        })
}

/// Returns the stable key for a quantity.
fn quantity_key(quantity: Quantity) -> Result<&'static str, ApiError> {
    Ok(match quantity {
        Quantity::Temperature => "temperature",
        Quantity::TemperatureDifference => "temperatureDifference",
        Quantity::Pressure => "pressure",
        Quantity::FlowRate => "flowRate",
        Quantity::Composition => "composition",
        Quantity::Density => "density",
        Quantity::Viscosity => "viscosity",
        Quantity::Energy => "energy",
        Quantity::Power => "power",
        Quantity::HeatTransferCoefficient => "heatTransferCoefficient",
        Quantity::ThermalConductivity => "thermalConductivity",
        Quantity::HeatFlux => "heatFlux",
        Quantity::SpecificHeat => "specificHeat",
        Quantity::SpecificEnthalpy => "specificEnthalpy",
        _ => {
            return Err(ApiError::internal(format!(
                "unsupported quantity: {}",
                quantity.name()
            )));
        }
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Everything the interface needs to know about units and context inputs.
pub struct Catalogue {
    pub quantities: Vec<QuantityEntry>,
    pub context: Vec<ContextEntry>,
    pub max_significant_figures: usize,
    pub default_significant_figures: usize,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct QuantityEntry {
    pub key: &'static str,
    pub name: &'static str,
    pub units: Vec<UnitEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct UnitEntry {
    pub label: &'static str,
    pub aliases: &'static [&'static str],
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ContextEntry {
    pub key: ContextKey,
    /// The query-string parameter that holds this field.
    pub param: &'static str,
    /// The library's description of the item.
    pub description: &'static str,
    pub input: ContextInput,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
/// How a context item is entered.
pub enum ContextInput {
    /// A plain number in a fixed unit (`None` for a dimensionless ratio).
    Number { unit: Option<&'static str> },
    /// A number with a pressure unit chosen from `units`.
    Pressure { units: Vec<&'static str> },
    /// A number with a temperature unit chosen from `units`.
    Temperature { units: Vec<&'static str> },
}

/// Lists every quantity with its units, and every context input.
pub fn catalogue() -> Catalogue {
    let mut quantities: Vec<QuantityEntry> = Vec::new();
    for unit in Unit::all() {
        let Ok(key) = quantity_key(unit.quantity()) else {
            continue;
        };
        let entry = UnitEntry {
            label: unit.label(),
            aliases: unit.aliases(),
        };
        match quantities.last_mut() {
            Some(last) if last.key == key => last.units.push(entry),
            _ => quantities.push(QuantityEntry {
                key,
                name: unit.quantity().name(),
                units: vec![entry],
            }),
        }
    }
    let context = ContextKey::ALL
        .into_iter()
        .map(|key| ContextEntry {
            key,
            param: key.param(),
            description: key.item().description(),
            input: context_input(key),
        })
        .collect();
    Catalogue {
        quantities,
        context,
        max_significant_figures: MAX_SIGNIFICANT_FIGURES,
        default_significant_figures: DEFAULT_SIGNIFICANT_FIGURES,
    }
}

fn context_input(key: ContextKey) -> ContextInput {
    let pressure_units = |include_gauge: bool| {
        Unit::all()
            .into_iter()
            .filter_map(|unit| match unit {
                Unit::Pressure(pressure) if include_gauge || !pressure.is_gauge() => {
                    Some(unit.label())
                }
                _ => None,
            })
            .collect()
    };
    match key {
        ContextKey::Density | ContextKey::ReferenceDensity => ContextInput::Number {
            unit: Some("kg/m³"),
        },
        ContextKey::MolarMass | ContextKey::SolventMolarMass => ContextInput::Number {
            unit: Some("g/mol"),
        },
        ContextKey::CompressibilityFactor => ContextInput::Number { unit: None },
        ContextKey::AtmosphericPressure => ContextInput::Pressure {
            units: pressure_units(false),
        },
        ContextKey::ReferencePressure | ContextKey::Pressure => ContextInput::Pressure {
            units: pressure_units(true),
        },
        ContextKey::ReferenceTemperature | ContextKey::Temperature => ContextInput::Temperature {
            units: Unit::all()
                .into_iter()
                .filter(|unit| matches!(unit, Unit::Temperature(_)))
                .map(Unit::label)
                .collect(),
        },
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ResolveResponse {
    Resolved {
        ok: bool,
        label: &'static str,
        quantity: &'static str,
    },
    Unresolved {
        ok: bool,
        error: ApiError,
    },
}

/// Resolves a typed unit name to its label and quantity.
pub fn resolve_unit(text: &str) -> ResolveResponse {
    let resolved = unit(text, InputField::From, "enter a unit")
        .and_then(|unit| Ok((unit.label(), quantity_key(unit.quantity())?)));
    match resolved {
        Ok((label, quantity)) => ResolveResponse::Resolved {
            ok: true,
            label,
            quantity,
        },
        Err(error) => ResolveResponse::Unresolved { ok: false, error },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::QuantityInput;

    fn request(value: &str, from: &str, to: &str) -> ConvertRequest {
        ConvertRequest {
            value: value.into(),
            from: from.into(),
            to: to.into(),
            ..ConvertRequest::default()
        }
    }

    fn quantity(value: &str, unit: &str) -> Option<QuantityInput> {
        Some(QuantityInput {
            value: value.into(),
            unit: unit.into(),
        })
    }

    fn failure(response: ConvertResponse) -> ApiError {
        match response {
            ConvertResponse::Failure(failure) => failure.error,
            ConvertResponse::Success(success) => panic!("expected an error, got {success:?}"),
        }
    }

    fn success(response: ConvertResponse) -> ConvertSuccess {
        match response {
            ConvertResponse::Success(success) => success,
            ConvertResponse::Failure(failure) => panic!("expected success, got {failure:?}"),
        }
    }

    #[test]
    fn converts_and_labels_the_result() {
        let result = success(convert(&request("25", "C", "F")));
        assert_eq!(result.formatted, "77");
        assert_eq!((result.from, result.to), ("°C", "°F"));
        assert_eq!(result.quantity, "temperature");
        assert!(result.used.is_empty());
    }

    #[test]
    fn missing_context_names_the_field_to_reveal() {
        let error = failure(convert(&request("30", "psig", "bar")));
        assert_eq!(error.kind, ErrorKind::MissingContext);
        assert_eq!(error.field, Field::Context(ContextKey::AtmosphericPressure));
        assert_eq!(error.missing, Some(ContextKey::AtmosphericPressure));
        assert_eq!(
            error.message,
            "converting between gauge and absolute pressure requires the local atmospheric pressure"
        );
    }

    #[test]
    fn gauge_context_pressure_reports_missing_atmosphere_against_itself() {
        let mut request = request("1000", "Sm3/h", "m3/h");
        request.context.pressure = quantity("5", "barg");
        let error = failure(convert(&request));
        assert_eq!(error.field, Field::Context(ContextKey::Pressure));
        assert_eq!(error.missing, Some(ContextKey::AtmosphericPressure));
    }

    #[test]
    fn reports_only_the_context_actually_used() {
        let mut request = request("1", "m3/s", "kg/h");
        request.context.density = Some("1000".into());
        request.context.molar_mass = Some("18".into());
        let result = success(convert(&request));
        assert_eq!(result.used, [ContextKey::Density]);
    }

    #[test]
    fn errors_point_at_the_responsible_input() {
        let cases = [
            (
                request("", "C", "F"),
                ErrorKind::EmptyInput,
                Field::Input(InputField::Value),
            ),
            (
                request("abc", "C", "F"),
                ErrorKind::InvalidNumber,
                Field::Input(InputField::Value),
            ),
            (
                request("1", "furlong", "F"),
                ErrorKind::UnrecognizedUnit,
                Field::Input(InputField::From),
            ),
            (
                request("1", "C", "mPa"),
                ErrorKind::AmbiguousPrefix,
                Field::Input(InputField::To),
            ),
            (
                request("1", "bar", "kg/h"),
                ErrorKind::IncompatibleUnits,
                Field::Input(InputField::To),
            ),
            (
                request("-1", "kg/h", "kg/s"),
                ErrorKind::NegativeValue,
                Field::Input(InputField::Value),
            ),
            (
                request("-500", "C", "K"),
                ErrorKind::BelowAbsoluteZero,
                Field::Input(InputField::Value),
            ),
        ];
        for (request, kind, field) in cases {
            let error = failure(convert(&request));
            assert_eq!((error.kind, error.field), (kind, field), "{request:?}");
        }
    }

    #[test]
    fn validates_context_fields_and_sig_figs() {
        let mut bad_density = request("1", "cP", "cSt");
        bad_density.context.density = Some("0".into());
        let error = failure(convert(&bad_density));
        assert_eq!(error.kind, ErrorKind::InvalidContext);
        assert_eq!(error.field, Field::Context(ContextKey::Density));

        let mut gauge_atmosphere = request("1", "psig", "psi");
        gauge_atmosphere.context.atmospheric_pressure = quantity("1", "psig");
        let error = failure(convert(&gauge_atmosphere));
        assert_eq!(error.kind, ErrorKind::WrongContextUnit);

        let mut no_unit = request("30", "psig", "bar");
        no_unit.context.atmospheric_pressure = quantity("101.325", " ");
        let error = failure(convert(&no_unit));
        assert_eq!(error.kind, ErrorKind::EmptyInput);
        assert_eq!(error.field, Field::Context(ContextKey::AtmosphericPressure));

        let mut sig_figs = request("1", "C", "F");
        sig_figs.sig_figs = Some("18".into());
        assert_eq!(failure(convert(&sig_figs)).kind, ErrorKind::InvalidSigFigs);
        sig_figs.sig_figs = Some(" ".into());
        assert_eq!(success(convert(&sig_figs)).formatted, "33.8");
    }

    #[test]
    fn catalogue_covers_every_unit_and_context_item() {
        let catalogue = catalogue();
        assert_eq!(catalogue.quantities.len(), 14);
        let units: usize = catalogue.quantities.iter().map(|q| q.units.len()).sum();
        assert_eq!(units, Unit::all().len());
        assert_eq!(catalogue.context.len(), ContextKey::ALL.len());
        // The web interface reserves v, q, from, to and sf for the main inputs.
        let mut params: Vec<&str> = catalogue.context.iter().map(|entry| entry.param).collect();
        params.extend(["v", "q", "from", "to", "sf"]);
        let count = params.len();
        params.sort_unstable();
        params.dedup();
        assert_eq!(params.len(), count, "query-string names must be unique");
        let atmosphere = catalogue
            .context
            .iter()
            .find(|entry| entry.key == ContextKey::AtmosphericPressure);
        let Some(ContextEntry {
            input: ContextInput::Pressure { units },
            ..
        }) = atmosphere
        else {
            panic!("atmospheric pressure should take a pressure unit");
        };
        assert!(units.contains(&"kPa") && !units.contains(&"psig"));
    }

    #[test]
    fn resolves_aliases_to_labels() {
        assert_eq!(
            resolve_unit("degC"),
            ResolveResponse::Resolved {
                ok: true,
                label: "°C",
                quantity: "temperature"
            }
        );
        assert!(matches!(
            resolve_unit("furlong"),
            ResolveResponse::Unresolved { ok: false, .. }
        ));
    }
}
