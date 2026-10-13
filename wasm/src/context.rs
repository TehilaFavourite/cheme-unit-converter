//! Turns the context fields of a request into a [`ConversionContext`],
//! following the same rules as the command-line interface.

use cheme_unit_converter::{
    ConversionContext, Pressure, PressureUnit, Temperature, TemperatureUnit, Unit, parse_unit,
};
use serde::{Deserialize, Serialize};

use crate::api::{ApiError, ErrorKind, Field};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
/// Stable names for the context inputs, shared with the web interface.
pub enum ContextKey {
    Density,
    MolarMass,
    SolventMolarMass,
    ReferenceDensity,
    AtmosphericPressure,
    ReferenceTemperature,
    ReferencePressure,
    Temperature,
    Pressure,
    CompressibilityFactor,
}

impl ContextKey {
    /// Every key, in the order the interface lists the fields.
    pub const ALL: [Self; 10] = [
        Self::Density,
        Self::MolarMass,
        Self::SolventMolarMass,
        Self::ReferenceDensity,
        Self::AtmosphericPressure,
        Self::ReferenceTemperature,
        Self::ReferencePressure,
        Self::Temperature,
        Self::Pressure,
        Self::CompressibilityFactor,
    ];

    /// Returns the key for a library context item, or `None` for an item
    /// this crate does not know yet.
    pub fn from_item(item: cheme_unit_converter::ContextItem) -> Option<Self> {
        use cheme_unit_converter::ContextItem;
        Some(match item {
            ContextItem::Density => Self::Density,
            ContextItem::MolarMass => Self::MolarMass,
            ContextItem::SolventMolarMass => Self::SolventMolarMass,
            ContextItem::ReferenceDensity => Self::ReferenceDensity,
            ContextItem::AtmosphericPressure => Self::AtmosphericPressure,
            ContextItem::ReferenceTemperature => Self::ReferenceTemperature,
            ContextItem::ReferencePressure => Self::ReferencePressure,
            ContextItem::OperatingTemperature => Self::Temperature,
            ContextItem::OperatingPressure => Self::Pressure,
            ContextItem::CompressibilityFactor => Self::CompressibilityFactor,
            _ => return None,
        })
    }

    /// Returns the library item this key supplies.
    pub fn item(self) -> cheme_unit_converter::ContextItem {
        use cheme_unit_converter::ContextItem;
        match self {
            Self::Density => ContextItem::Density,
            Self::MolarMass => ContextItem::MolarMass,
            Self::SolventMolarMass => ContextItem::SolventMolarMass,
            Self::ReferenceDensity => ContextItem::ReferenceDensity,
            Self::AtmosphericPressure => ContextItem::AtmosphericPressure,
            Self::ReferenceTemperature => ContextItem::ReferenceTemperature,
            Self::ReferencePressure => ContextItem::ReferencePressure,
            Self::Temperature => ContextItem::OperatingTemperature,
            Self::Pressure => ContextItem::OperatingPressure,
            Self::CompressibilityFactor => ContextItem::CompressibilityFactor,
        }
    }
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
/// Context fields as typed in the interface. Blank fields count as absent.
pub struct ContextRequest {
    #[serde(default)]
    pub density: Option<String>,
    #[serde(default)]
    pub molar_mass: Option<String>,
    #[serde(default)]
    pub solvent_molar_mass: Option<String>,
    #[serde(default)]
    pub reference_density: Option<String>,
    #[serde(default)]
    pub compressibility_factor: Option<String>,
    #[serde(default)]
    pub atmospheric_pressure: Option<QuantityInput>,
    #[serde(default)]
    pub reference_temperature: Option<QuantityInput>,
    #[serde(default)]
    pub reference_pressure: Option<QuantityInput>,
    #[serde(default)]
    pub temperature: Option<QuantityInput>,
    #[serde(default)]
    pub pressure: Option<QuantityInput>,
}

#[derive(Debug, Clone, Deserialize)]
/// A context value that is itself a quantity: a number and a unit name.
pub struct QuantityInput {
    pub value: String,
    pub unit: String,
}

/// Context fields after parsing, before they are combined into a
/// [`ConversionContext`].
#[derive(Debug, Clone, Copy, Default)]
pub struct ContextInputs {
    density: Option<f64>,
    molar_mass: Option<f64>,
    solvent_molar_mass: Option<f64>,
    reference_density: Option<f64>,
    compressibility_factor: Option<f64>,
    atmospheric_pressure: Option<(f64, PressureUnit)>,
    reference_temperature: Option<(f64, TemperatureUnit)>,
    reference_pressure: Option<(f64, PressureUnit)>,
    temperature: Option<(f64, TemperatureUnit)>,
    pressure: Option<(f64, PressureUnit)>,
}

impl ContextInputs {
    /// Parses every supplied field, reporting the first one that is invalid.
    pub fn parse(request: &ContextRequest) -> Result<Self, ApiError> {
        Ok(Self {
            density: number(request.density.as_deref(), ContextKey::Density)?,
            molar_mass: number(request.molar_mass.as_deref(), ContextKey::MolarMass)?,
            solvent_molar_mass: number(
                request.solvent_molar_mass.as_deref(),
                ContextKey::SolventMolarMass,
            )?,
            reference_density: number(
                request.reference_density.as_deref(),
                ContextKey::ReferenceDensity,
            )?,
            compressibility_factor: number(
                request.compressibility_factor.as_deref(),
                ContextKey::CompressibilityFactor,
            )?,
            atmospheric_pressure: atmospheric_pressure(request.atmospheric_pressure.as_ref())?,
            reference_temperature: temperature(
                request.reference_temperature.as_ref(),
                ContextKey::ReferenceTemperature,
            )?,
            reference_pressure: pressure(
                request.reference_pressure.as_ref(),
                ContextKey::ReferencePressure,
            )?,
            temperature: temperature(request.temperature.as_ref(), ContextKey::Temperature)?,
            pressure: pressure(request.pressure.as_ref(), ContextKey::Pressure)?,
        })
    }

    /// Returns the keys of the fields that were supplied.
    pub fn supplied(&self) -> Vec<ContextKey> {
        ContextKey::ALL
            .into_iter()
            .filter(|&key| self.has(key))
            .collect()
    }

    /// Returns a copy with `key` removed.
    pub fn without(mut self, key: ContextKey) -> Self {
        match key {
            ContextKey::Density => self.density = None,
            ContextKey::MolarMass => self.molar_mass = None,
            ContextKey::SolventMolarMass => self.solvent_molar_mass = None,
            ContextKey::ReferenceDensity => self.reference_density = None,
            ContextKey::CompressibilityFactor => self.compressibility_factor = None,
            ContextKey::AtmosphericPressure => self.atmospheric_pressure = None,
            ContextKey::ReferenceTemperature => self.reference_temperature = None,
            ContextKey::ReferencePressure => self.reference_pressure = None,
            ContextKey::Temperature => self.temperature = None,
            ContextKey::Pressure => self.pressure = None,
        }
        self
    }

    fn has(&self, key: ContextKey) -> bool {
        match key {
            ContextKey::Density => self.density.is_some(),
            ContextKey::MolarMass => self.molar_mass.is_some(),
            ContextKey::SolventMolarMass => self.solvent_molar_mass.is_some(),
            ContextKey::ReferenceDensity => self.reference_density.is_some(),
            ContextKey::CompressibilityFactor => self.compressibility_factor.is_some(),
            ContextKey::AtmosphericPressure => self.atmospheric_pressure.is_some(),
            ContextKey::ReferenceTemperature => self.reference_temperature.is_some(),
            ContextKey::ReferencePressure => self.reference_pressure.is_some(),
            ContextKey::Temperature => self.temperature.is_some(),
            ContextKey::Pressure => self.pressure.is_some(),
        }
    }

    /// Builds the library context. The atmospheric pressure is built first
    /// so that gauge context pressures can use it, as the CLI does.
    pub fn build(&self) -> Result<ConversionContext, ApiError> {
        let mut context = ConversionContext::new();
        context.density_kg_m3 = self.density;
        context.molar_mass_g_mol = self.molar_mass;
        context.solvent_molar_mass_g_mol = self.solvent_molar_mass;
        context.reference_density_kg_m3 = self.reference_density;
        context.compressibility_factor = self.compressibility_factor;
        context.reference_temperature =
            build_temperature(self.reference_temperature, ContextKey::ReferenceTemperature)?;
        context.temperature = build_temperature(self.temperature, ContextKey::Temperature)?;
        context.atmospheric_pressure = build_pressure(
            self.atmospheric_pressure,
            ContextKey::AtmosphericPressure,
            &context,
        )?;
        context.reference_pressure = build_pressure(
            self.reference_pressure,
            ContextKey::ReferencePressure,
            &context,
        )?;
        context.pressure = build_pressure(self.pressure, ContextKey::Pressure, &context)?;
        Ok(context)
    }
}

/// Returns the trimmed text, or `None` if it is absent or blank.
fn present(text: Option<&str>) -> Option<&str> {
    text.map(str::trim).filter(|text| !text.is_empty())
}

/// Parses a number exactly as the CLI does.
pub fn parse_number(text: &str, field: Field) -> Result<f64, ApiError> {
    let text = text.trim();
    text.parse::<f64>().map_err(|_| {
        ApiError::new(
            ErrorKind::InvalidNumber,
            field,
            format!("invalid numeric value '{text}'"),
        )
    })
}

fn number(text: Option<&str>, key: ContextKey) -> Result<Option<f64>, ApiError> {
    present(text)
        .map(|text| parse_number(text, Field::Context(key)))
        .transpose()
}

/// Parses a value and unit pair, returning `None` when the value is blank.
fn value_and_unit(
    input: Option<&QuantityInput>,
    key: ContextKey,
) -> Result<Option<(f64, Unit)>, ApiError> {
    let Some(input) = input else {
        return Ok(None);
    };
    let Some(value) = present(Some(&input.value)) else {
        return Ok(None);
    };
    let value = parse_number(value, Field::Context(key))?;
    let unit =
        parse_unit(&input.unit).map_err(|error| ApiError::unit(&error, Field::Context(key)))?;
    Ok(Some((value, unit)))
}

fn pressure(
    input: Option<&QuantityInput>,
    key: ContextKey,
) -> Result<Option<(f64, PressureUnit)>, ApiError> {
    match value_and_unit(input, key)? {
        None => Ok(None),
        Some((value, Unit::Pressure(unit))) => Ok(Some((value, unit))),
        Some(_) => Err(wrong_unit(key, "a pressure unit such as kPa, bar, or psi")),
    }
}

fn atmospheric_pressure(
    input: Option<&QuantityInput>,
) -> Result<Option<(f64, PressureUnit)>, ApiError> {
    let key = ContextKey::AtmosphericPressure;
    match pressure(input, key)? {
        Some((_, unit)) if unit.is_gauge() => Err(wrong_unit(key, "an absolute pressure unit")),
        parsed => Ok(parsed),
    }
}

fn temperature(
    input: Option<&QuantityInput>,
    key: ContextKey,
) -> Result<Option<(f64, TemperatureUnit)>, ApiError> {
    match value_and_unit(input, key)? {
        None => Ok(None),
        Some((value, Unit::Temperature(unit))) => Ok(Some((value, unit))),
        Some(_) => Err(wrong_unit(key, "a temperature unit such as C, F, K, or R")),
    }
}

fn wrong_unit(key: ContextKey, expected: &str) -> ApiError {
    ApiError::new(
        ErrorKind::WrongContextUnit,
        Field::Context(key),
        format!("the {} needs {expected}", key.item().description()),
    )
}

fn build_pressure(
    input: Option<(f64, PressureUnit)>,
    key: ContextKey,
    context: &ConversionContext,
) -> Result<Option<Pressure>, ApiError> {
    input
        .map(|(value, unit)| Pressure::new(value, unit, context))
        .transpose()
        .map_err(|error| ApiError::conversion(&error, Field::Context(key)))
}

fn build_temperature(
    input: Option<(f64, TemperatureUnit)>,
    key: ContextKey,
) -> Result<Option<Temperature>, ApiError> {
    input
        .map(|(value, unit)| Temperature::new(value, unit))
        .transpose()
        .map_err(|error| ApiError::conversion(&error, Field::Context(key)))
}
