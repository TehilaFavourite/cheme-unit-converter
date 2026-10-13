//! Errors, quantity names, and the context items that conversions can need.

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// The physical quantity that a unit measures.
pub enum Quantity {
    /// A temperature reading on a scale (°C, °F, K, °R).
    Temperature,
    /// A temperature interval (Δ°C, Δ°F, ΔK, Δ°R).
    TemperatureDifference,
    /// Absolute or gauge pressure.
    Pressure,
    /// Mass, volumetric, molar, or standard gas flow.
    FlowRate,
    /// Solution or mixture composition.
    Composition,
    /// Density, specific gravity, or API gravity.
    Density,
    /// Dynamic or kinematic viscosity.
    Viscosity,
    /// Energy or heat.
    Energy,
    /// Power or heat duty.
    Power,
    /// Heat transfer (film or overall) coefficient.
    HeatTransferCoefficient,
    /// Thermal conductivity.
    ThermalConductivity,
    /// Heat flux.
    HeatFlux,
    /// Mass or molar specific heat capacity.
    SpecificHeat,
    /// Mass or molar specific enthalpy.
    SpecificEnthalpy,
}

impl Quantity {
    /// Returns a lowercase name suitable for messages.
    pub fn name(self) -> &'static str {
        match self {
            Self::Temperature => "temperature",
            Self::TemperatureDifference => "temperature difference",
            Self::Pressure => "pressure",
            Self::FlowRate => "flow rate",
            Self::Composition => "composition",
            Self::Density => "density",
            Self::Viscosity => "viscosity",
            Self::Energy => "energy",
            Self::Power => "power",
            Self::HeatTransferCoefficient => "heat transfer coefficient",
            Self::ThermalConductivity => "thermal conductivity",
            Self::HeatFlux => "heat flux",
            Self::SpecificHeat => "specific heat capacity",
            Self::SpecificEnthalpy => "specific enthalpy",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// A physical property or reference condition that a conversion depends on.
///
/// The converter never assumes a value for any of these. A conversion that
/// needs one fails with [`ConversionError::MissingContext`] instead.
pub enum ContextItem {
    /// [`ConversionContext::density_kg_m3`](crate::ConversionContext::density_kg_m3).
    Density,
    /// [`ConversionContext::molar_mass_g_mol`](crate::ConversionContext::molar_mass_g_mol).
    MolarMass,
    /// [`ConversionContext::solvent_molar_mass_g_mol`](crate::ConversionContext::solvent_molar_mass_g_mol).
    SolventMolarMass,
    /// [`ConversionContext::reference_density_kg_m3`](crate::ConversionContext::reference_density_kg_m3).
    ReferenceDensity,
    /// [`ConversionContext::atmospheric_pressure`](crate::ConversionContext::atmospheric_pressure).
    AtmosphericPressure,
    /// [`ConversionContext::reference_temperature`](crate::ConversionContext::reference_temperature).
    ReferenceTemperature,
    /// [`ConversionContext::reference_pressure`](crate::ConversionContext::reference_pressure).
    ReferencePressure,
    /// [`ConversionContext::temperature`](crate::ConversionContext::temperature).
    OperatingTemperature,
    /// [`ConversionContext::pressure`](crate::ConversionContext::pressure).
    OperatingPressure,
    /// [`ConversionContext::compressibility_factor`](crate::ConversionContext::compressibility_factor).
    CompressibilityFactor,
}

impl ContextItem {
    /// Describes the item, including its expected unit where it has a fixed one.
    pub fn description(self) -> &'static str {
        match self {
            Self::Density => "fluid density (kg/m³)",
            Self::MolarMass => "molar mass of the substance or solute (g/mol)",
            Self::SolventMolarMass => "molar mass of the solvent or rest of the mixture (g/mol)",
            Self::ReferenceDensity => "reference density for specific gravity (kg/m³)",
            Self::AtmosphericPressure => "local atmospheric pressure",
            Self::ReferenceTemperature => "standard reference temperature",
            Self::ReferencePressure => "standard reference pressure",
            Self::OperatingTemperature => "actual gas temperature",
            Self::OperatingPressure => "actual gas pressure",
            Self::CompressibilityFactor => "gas compressibility factor Z at the actual conditions",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[non_exhaustive]
/// An invalid value, unsupported conversion, or missing conversion context.
pub enum ConversionError {
    /// The input value is NaN or infinite.
    NonFiniteValue,
    /// The result is too large to represent as a finite `f64`.
    ResultOverflow,
    /// A temperature reading is below 0 K.
    BelowAbsoluteZero,
    /// A pressure is below zero absolute (for example, a vacuum gauge reading
    /// larger than the atmospheric pressure).
    NegativePressure,
    /// A quantity that cannot be negative received a negative value.
    NegativeValue(Quantity),
    /// The value is outside the valid range for the named unit, such as a
    /// fraction above 100%.
    OutOfRange(&'static str),
    /// The conversion needs a property or reference condition that the
    /// context does not supply.
    MissingContext {
        /// The missing property or condition.
        item: ContextItem,
        /// The conversion step that needs it.
        reason: &'static str,
    },
    /// A supplied property or condition is zero, negative, or not finite.
    InvalidContext(ContextItem),
    /// The units measure different quantities.
    IncompatibleUnits {
        /// The quantity of the source unit.
        from: Quantity,
        /// The quantity of the target unit.
        to: Quantity,
    },
}

impl fmt::Display for ConversionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFiniteValue => write!(f, "value must be a finite number"),
            Self::ResultOverflow => write!(f, "result is too large to represent"),
            Self::BelowAbsoluteZero => write!(f, "temperature is below absolute zero"),
            Self::NegativePressure => write!(f, "absolute pressure cannot be negative"),
            Self::NegativeValue(quantity) => write!(f, "{} cannot be negative", quantity.name()),
            Self::OutOfRange(unit) => write!(f, "value is outside the valid range for {unit}"),
            Self::MissingContext { item, reason } => {
                write!(f, "{reason} requires the {}", item.description())
            }
            Self::InvalidContext(item) => {
                write!(
                    f,
                    "the {} must be a finite positive number",
                    item.description()
                )
            }
            Self::IncompatibleUnits { from, to } => {
                write!(f, "cannot convert {} to {}", from.name(), to.name())?;
                if matches!(
                    (from, to),
                    (Quantity::Temperature, Quantity::TemperatureDifference)
                        | (Quantity::TemperatureDifference, Quantity::Temperature)
                ) {
                    write!(
                        f,
                        "; C, F, K and R are temperature readings, while dC, dF, dK and dR \
                         are temperature differences"
                    )?;
                }
                if matches!(
                    (from, to),
                    (Quantity::Energy, Quantity::Power) | (Quantity::Power, Quantity::Energy)
                ) {
                    write!(
                        f,
                        "; power is energy per unit time, so relating them needs a time interval"
                    )?;
                }
                Ok(())
            }
        }
    }
}

impl std::error::Error for ConversionError {}

pub(crate) fn validate_finite(value: f64) -> Result<(), ConversionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ConversionError::NonFiniteValue)
    }
}

/// Checks a computed value, reporting overflow rather than a bad input.
pub(crate) fn validate_result(value: f64) -> Result<(), ConversionError> {
    if value.is_finite() {
        Ok(())
    } else {
        Err(ConversionError::ResultOverflow)
    }
}

pub(crate) fn validate_non_negative(value: f64, quantity: Quantity) -> Result<(), ConversionError> {
    validate_finite(value)?;
    if value < 0.0 {
        Err(ConversionError::NegativeValue(quantity))
    } else {
        Ok(())
    }
}

/// Whether a quantity may take negative values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Sign {
    /// Signed quantities such as heat duty or enthalpy relative to a reference state.
    Any,
    NonNegative,
}

pub(crate) fn validate_sign(
    value: f64,
    quantity: Quantity,
    sign: Sign,
) -> Result<(), ConversionError> {
    match sign {
        Sign::Any => validate_finite(value),
        Sign::NonNegative => validate_non_negative(value, quantity),
    }
}
