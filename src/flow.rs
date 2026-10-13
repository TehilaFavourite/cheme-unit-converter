//! Mass, volumetric, molar, and standard-volume flow rates.
//!
//! Every flow unit belongs to one basis. Converting within a basis only
//! rescales the value. Converting across bases needs the properties that
//! relate them:
//!
//! - mass ↔ volumetric: density
//! - mass ↔ molar: molar mass
//! - volumetric ↔ molar: density and molar mass
//! - standard volume ↔ molar: reference temperature and pressure (ideal gas)
//! - standard ↔ actual volume: reference and actual temperature and pressure,
//!   plus the compressibility factor Z at the actual conditions
//!
//! Standard volume is defined as ideal-gas volume at the reference
//! conditions, so it needs no compressibility factor. Actual volume is
//! V = Z·n·R·T / P; supply Z = 1 to treat the stream as an ideal gas.

use crate::constants::{
    CUBIC_METRES_PER_CUBIC_FOOT, CUBIC_METRES_PER_US_GALLON, GRAMS_PER_KILOGRAM,
    KILOGRAMS_PER_POUND, MOLAR_GAS_CONSTANT, MOLES_PER_POUND_MOLE, SECONDS_PER_HOUR,
};
use crate::context::ConversionContext;
use crate::error::{
    ContextItem, ConversionError, Quantity, validate_non_negative, validate_result,
};
use crate::unit::UnitSet;

const SECONDS_PER_MINUTE: f64 = 60.0;
const SECONDS_PER_DAY: f64 = 86_400.0;
/// US oil barrel: 42 US gallons.
const CUBIC_METRES_PER_BARREL: f64 = 42.0 * CUBIC_METRES_PER_US_GALLON;

const MASS_VOLUME: &str = "converting between mass and volumetric flow";
const MASS_MOLAR: &str =
    "converting mass flow to or from a mole-based flow (molar or standard gas)";
const VOLUME_MOLAR: &str = "converting between volumetric and molar flow";
const STANDARD_VOLUME: &str = "converting standard gas volume flow (Sm³/h, SCFM, ...)";
const ACTUAL_STANDARD: &str = "converting between actual and standard gas volume flow";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowBasis {
    /// kg/s.
    Mass,
    /// m³/s at the actual flowing conditions.
    Volume,
    /// mol/s.
    Molar,
    /// m³/s at the caller's reference temperature and pressure.
    StandardVolume,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Mass, volumetric, molar, and standard gas flow units.
pub enum FlowRateUnit {
    /// Kilograms per second (`kg/s`).
    KilogramsPerSecond,
    /// Kilograms per hour (`kg/h`).
    KilogramsPerHour,
    /// Grams per second (`g/s`).
    GramsPerSecond,
    /// Tonnes per hour (`t/h`).
    TonnesPerHour,
    /// Pounds per second (`lb/s`).
    PoundsPerSecond,
    /// Pounds per minute (`lb/min`).
    PoundsPerMinute,
    /// Pounds per hour (`lb/h`).
    PoundsPerHour,
    /// Cubic metres per second (`m³/s`).
    CubicMetresPerSecond,
    /// Cubic metres per hour (`m³/h`).
    CubicMetresPerHour,
    /// Litres per second (`L/s`).
    LitresPerSecond,
    /// Litres per minute (`L/min`).
    LitresPerMinute,
    /// Millilitres per minute (`mL/min`).
    MillilitresPerMinute,
    /// US gallons per minute (`GPM`).
    UsGallonsPerMinute,
    /// Cubic feet per minute (`ft³/min`).
    CubicFeetPerMinute,
    /// Barrels per day (`bbl/d`).
    BarrelsPerDay,
    /// Moles per second (`mol/s`).
    MolesPerSecond,
    /// Kilomoles per hour (`kmol/h`).
    KilomolesPerHour,
    /// Pound moles per hour (`lbmol/h`).
    PoundMolesPerHour,
    /// Standard cubic metres per hour (`Sm³/h`).
    StandardCubicMetresPerHour,
    /// Standard cubic feet per minute (`SCFM`).
    StandardCubicFeetPerMinute,
    /// Million standard cubic feet per day (`MMSCFD`).
    MillionStandardCubicFeetPerDay,
    /// Standard litres per minute (`SLPM`).
    StandardLitresPerMinute,
    /// Standard cubic centimetres per minute (`SCCM`).
    StandardCubicCentimetresPerMinute,
}

impl FlowRateUnit {
    /// Returns the unit's basis and the size of one unit in that basis's SI unit.
    fn basis_and_scale(self) -> (FlowBasis, f64) {
        use FlowBasis::{Mass, Molar, StandardVolume, Volume};
        match self {
            Self::KilogramsPerSecond => (Mass, 1.0),
            Self::KilogramsPerHour => (Mass, 1.0 / SECONDS_PER_HOUR),
            Self::GramsPerSecond => (Mass, 1.0 / GRAMS_PER_KILOGRAM),
            Self::TonnesPerHour => (Mass, 1_000.0 / SECONDS_PER_HOUR),
            Self::PoundsPerSecond => (Mass, KILOGRAMS_PER_POUND),
            Self::PoundsPerMinute => (Mass, KILOGRAMS_PER_POUND / SECONDS_PER_MINUTE),
            Self::PoundsPerHour => (Mass, KILOGRAMS_PER_POUND / SECONDS_PER_HOUR),
            Self::CubicMetresPerSecond => (Volume, 1.0),
            Self::CubicMetresPerHour => (Volume, 1.0 / SECONDS_PER_HOUR),
            Self::LitresPerSecond => (Volume, 1e-3),
            Self::LitresPerMinute => (Volume, 1e-3 / SECONDS_PER_MINUTE),
            Self::MillilitresPerMinute => (Volume, 1e-6 / SECONDS_PER_MINUTE),
            Self::UsGallonsPerMinute => (Volume, CUBIC_METRES_PER_US_GALLON / SECONDS_PER_MINUTE),
            Self::CubicFeetPerMinute => (Volume, CUBIC_METRES_PER_CUBIC_FOOT / SECONDS_PER_MINUTE),
            Self::BarrelsPerDay => (Volume, CUBIC_METRES_PER_BARREL / SECONDS_PER_DAY),
            Self::MolesPerSecond => (Molar, 1.0),
            Self::KilomolesPerHour => (Molar, 1_000.0 / SECONDS_PER_HOUR),
            Self::PoundMolesPerHour => (Molar, MOLES_PER_POUND_MOLE / SECONDS_PER_HOUR),
            Self::StandardCubicMetresPerHour => (StandardVolume, 1.0 / SECONDS_PER_HOUR),
            Self::StandardCubicFeetPerMinute => (
                StandardVolume,
                CUBIC_METRES_PER_CUBIC_FOOT / SECONDS_PER_MINUTE,
            ),
            Self::MillionStandardCubicFeetPerDay => (
                StandardVolume,
                1e6 * CUBIC_METRES_PER_CUBIC_FOOT / SECONDS_PER_DAY,
            ),
            Self::StandardLitresPerMinute => (StandardVolume, 1e-3 / SECONDS_PER_MINUTE),
            Self::StandardCubicCentimetresPerMinute => (StandardVolume, 1e-6 / SECONDS_PER_MINUTE),
        }
    }
}

impl UnitSet for FlowRateUnit {
    const ALL: &'static [Self] = &[
        Self::KilogramsPerSecond,
        Self::KilogramsPerHour,
        Self::GramsPerSecond,
        Self::TonnesPerHour,
        Self::PoundsPerSecond,
        Self::PoundsPerMinute,
        Self::PoundsPerHour,
        Self::CubicMetresPerSecond,
        Self::CubicMetresPerHour,
        Self::LitresPerSecond,
        Self::LitresPerMinute,
        Self::MillilitresPerMinute,
        Self::UsGallonsPerMinute,
        Self::CubicFeetPerMinute,
        Self::BarrelsPerDay,
        Self::MolesPerSecond,
        Self::KilomolesPerHour,
        Self::PoundMolesPerHour,
        Self::StandardCubicMetresPerHour,
        Self::StandardCubicFeetPerMinute,
        Self::MillionStandardCubicFeetPerDay,
        Self::StandardLitresPerMinute,
        Self::StandardCubicCentimetresPerMinute,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::KilogramsPerSecond => "kg/s",
            Self::KilogramsPerHour => "kg/h",
            Self::GramsPerSecond => "g/s",
            Self::TonnesPerHour => "t/h",
            Self::PoundsPerSecond => "lb/s",
            Self::PoundsPerMinute => "lb/min",
            Self::PoundsPerHour => "lb/h",
            Self::CubicMetresPerSecond => "m³/s",
            Self::CubicMetresPerHour => "m³/h",
            Self::LitresPerSecond => "L/s",
            Self::LitresPerMinute => "L/min",
            Self::MillilitresPerMinute => "mL/min",
            Self::UsGallonsPerMinute => "GPM",
            Self::CubicFeetPerMinute => "ft³/min",
            Self::BarrelsPerDay => "bbl/d",
            Self::MolesPerSecond => "mol/s",
            Self::KilomolesPerHour => "kmol/h",
            Self::PoundMolesPerHour => "lbmol/h",
            Self::StandardCubicMetresPerHour => "Sm³/h",
            Self::StandardCubicFeetPerMinute => "SCFM",
            Self::MillionStandardCubicFeetPerDay => "MMSCFD",
            Self::StandardLitresPerMinute => "SLPM",
            Self::StandardCubicCentimetresPerMinute => "SCCM",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::KilogramsPerSecond => &[],
            Self::KilogramsPerHour => &[],
            Self::GramsPerSecond => &[],
            Self::TonnesPerHour => &["tonne/h"],
            Self::PoundsPerSecond => &["lbs/s"],
            Self::PoundsPerMinute => &["lbs/min"],
            Self::PoundsPerHour => &["lbs/h"],
            Self::CubicMetresPerSecond => &[],
            Self::CubicMetresPerHour => &[],
            Self::LitresPerSecond => &[],
            Self::LitresPerMinute => &["lpm"],
            Self::MillilitresPerMinute => &[],
            Self::UsGallonsPerMinute => &["usgpm"],
            Self::CubicFeetPerMinute => &["cfm", "acfm"],
            Self::BarrelsPerDay => &["bbl/day", "bpd"],
            Self::MolesPerSecond => &[],
            Self::KilomolesPerHour => &[],
            Self::PoundMolesPerHour => &[],
            Self::StandardCubicMetresPerHour => &["nm³/h"],
            Self::StandardCubicFeetPerMinute => &[],
            Self::MillionStandardCubicFeetPerDay => &[],
            Self::StandardLitresPerMinute => &["sl/min"],
            Self::StandardCubicCentimetresPerMinute => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A non-negative flow rate stored in the SI unit of its own basis.
pub struct FlowRate {
    basis: FlowBasis,
    value: f64,
}

impl FlowRate {
    /// Creates a flow rate from a value and its source unit.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is not finite or is outside the valid range
    /// for `unit`.
    pub fn new(value: f64, unit: FlowRateUnit) -> Result<Self, ConversionError> {
        validate_non_negative(value, Quantity::FlowRate)?;
        let (basis, scale) = unit.basis_and_scale();
        let value = value * scale;
        validate_result(value)?;
        Ok(Self { basis, value })
    }

    /// Returns this flow rate in `unit`.
    ///
    /// Fails with [`ConversionError::MissingContext`] when `unit` has a
    /// different basis and `context` lacks a property needed to relate them.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    pub fn in_unit(
        self,
        unit: FlowRateUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let (basis, scale) = unit.basis_and_scale();
        let value = convert_basis(self.value, self.basis, basis, context)? / scale;
        validate_result(value)?;
        Ok(value)
    }
}

fn convert_basis(
    value: f64,
    from: FlowBasis,
    to: FlowBasis,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    use FlowBasis::{Mass, StandardVolume, Volume};
    match (from, to) {
        _ if from == to => Ok(value),
        (Mass, Volume) => Ok(value / context.require(ContextItem::Density, MASS_VOLUME)?),
        (Volume, Mass) => Ok(value * context.require(ContextItem::Density, MASS_VOLUME)?),
        (Volume, StandardVolume) => Ok(value * standard_per_actual_volume(context)?),
        (StandardVolume, Volume) => Ok(value / standard_per_actual_volume(context)?),
        _ => from_moles(to_moles(value, from, context)?, to, context),
    }
}

/// Ratio V_std / V_actual = (P / P_ref) · (T_ref / T) / Z.
fn standard_per_actual_volume(context: &ConversionContext) -> Result<f64, ConversionError> {
    let pressure = context.require(ContextItem::OperatingPressure, ACTUAL_STANDARD)?;
    let temperature = context.require(ContextItem::OperatingTemperature, ACTUAL_STANDARD)?;
    let reference_pressure = context.require(ContextItem::ReferencePressure, ACTUAL_STANDARD)?;
    let reference_temperature =
        context.require(ContextItem::ReferenceTemperature, ACTUAL_STANDARD)?;
    let compressibility = context.require(ContextItem::CompressibilityFactor, ACTUAL_STANDARD)?;
    Ok((pressure / reference_pressure) * (reference_temperature / temperature) / compressibility)
}

/// Ideal-gas moles per cubic metre at the reference conditions, P / (R·T).
fn standard_molar_density(context: &ConversionContext) -> Result<f64, ConversionError> {
    let pressure = context.require(ContextItem::ReferencePressure, STANDARD_VOLUME)?;
    let temperature = context.require(ContextItem::ReferenceTemperature, STANDARD_VOLUME)?;
    Ok(pressure / (MOLAR_GAS_CONSTANT * temperature))
}

fn to_moles(
    value: f64,
    basis: FlowBasis,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    Ok(match basis {
        FlowBasis::Molar => value,
        FlowBasis::Mass => {
            value * GRAMS_PER_KILOGRAM / context.require(ContextItem::MolarMass, MASS_MOLAR)?
        }
        FlowBasis::Volume => {
            value * context.require(ContextItem::Density, VOLUME_MOLAR)? * GRAMS_PER_KILOGRAM
                / context.require(ContextItem::MolarMass, VOLUME_MOLAR)?
        }
        FlowBasis::StandardVolume => value * standard_molar_density(context)?,
    })
}

fn from_moles(
    moles: f64,
    basis: FlowBasis,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    Ok(match basis {
        FlowBasis::Molar => moles,
        FlowBasis::Mass => {
            moles * context.require(ContextItem::MolarMass, MASS_MOLAR)? / GRAMS_PER_KILOGRAM
        }
        FlowBasis::Volume => {
            moles * context.require(ContextItem::MolarMass, VOLUME_MOLAR)?
                / GRAMS_PER_KILOGRAM
                / context.require(ContextItem::Density, VOLUME_MOLAR)?
        }
        FlowBasis::StandardVolume => moles / standard_molar_density(context)?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pressure::{Pressure, PressureUnit};
    use crate::temperature::{Temperature, TemperatureUnit};
    use crate::test_support::{assert_close, convert_str};

    fn standard_conditions() -> ConversionContext {
        let none = ConversionContext::default();
        ConversionContext {
            reference_temperature: Some(Temperature::new(0.0, TemperatureUnit::Celsius).unwrap()),
            reference_pressure: Some(Pressure::new(1.0, PressureUnit::Atmosphere, &none).unwrap()),
            ..none
        }
    }

    #[test]
    fn rescales_within_a_basis_without_context() {
        let none = ConversionContext::default();
        assert_close(
            convert_str(3_600.0, "kg/h", "lb/min", none).unwrap(),
            132.277_357_310_9,
        );
        assert_close(convert_str(1.0, "L/min", "mL/min", none).unwrap(), 1_000.0);
        assert_close(
            convert_str(1.0, "GPM", "m3/h", none).unwrap(),
            0.227_124_707_04,
        );
        assert_close(
            convert_str(1.0, "kmol/h", "mol/s", none).unwrap(),
            1.0 / 3.6,
        );
        assert_close(convert_str(1.0, "SLPM", "SCCM", none).unwrap(), 1_000.0);
    }

    #[test]
    fn mass_and_volume_flow_need_density() {
        assert_eq!(
            convert_str(1.0, "m3/s", "kg/h", ConversionContext::default()),
            Err(ConversionError::MissingContext {
                item: ContextItem::Density,
                reason: MASS_VOLUME,
            })
        );
        let water = ConversionContext {
            density_kg_m3: Some(1_000.0),
            ..ConversionContext::default()
        };
        assert_close(
            convert_str(1.0, "m3/s", "kg/h", water).unwrap(),
            3_600_000.0,
        );
    }

    #[test]
    fn mass_and_molar_flow_need_molar_mass() {
        let water = ConversionContext {
            molar_mass_g_mol: Some(18.015),
            ..ConversionContext::default()
        };
        assert_close(convert_str(1.0, "kmol/h", "kg/h", water).unwrap(), 18.015);
        assert_close(convert_str(18.015, "lb/h", "lbmol/h", water).unwrap(), 1.0);
    }

    #[test]
    fn standard_volume_uses_stated_reference_conditions() {
        assert!(matches!(
            convert_str(1.0, "kmol/h", "Sm3/h", ConversionContext::default()),
            Err(ConversionError::MissingContext {
                item: ContextItem::ReferencePressure,
                ..
            })
        ));
        // Ideal-gas molar volume at 0 °C and 1 atm is 22.414 L/mol.
        let result = convert_str(1.0, "kmol/h", "Sm3/h", standard_conditions()).unwrap();
        assert_close(result, 22.413_969_545_014_14);
    }

    #[test]
    fn converts_between_actual_and_standard_volume() {
        let none = ConversionContext::default();
        let context = ConversionContext {
            temperature: Some(Temperature::new(546.3, TemperatureUnit::Kelvin).unwrap()),
            pressure: Some(Pressure::new(2.0, PressureUnit::Atmosphere, &none).unwrap()),
            compressibility_factor: Some(1.0),
            ..standard_conditions()
        };
        // Doubling both absolute temperature and pressure leaves the volume unchanged.
        assert_close(convert_str(100.0, "Sm3/h", "m3/h", context).unwrap(), 100.0);
        // A real gas with Z = 0.9 occupies 90% of the ideal-gas volume.
        let real_gas = ConversionContext {
            compressibility_factor: Some(0.9),
            ..context
        };
        assert_close(convert_str(100.0, "Sm3/h", "m3/h", real_gas).unwrap(), 90.0);
        assert!(matches!(
            convert_str(
                100.0,
                "Sm3/h",
                "m3/h",
                ConversionContext {
                    compressibility_factor: None,
                    ..context
                }
            ),
            Err(ConversionError::MissingContext {
                item: ContextItem::CompressibilityFactor,
                ..
            })
        ));
        assert!(matches!(
            convert_str(100.0, "Sm3/h", "m3/h", standard_conditions()),
            Err(ConversionError::MissingContext {
                item: ContextItem::OperatingPressure,
                ..
            })
        ));
    }

    #[test]
    fn rejects_negative_flow() {
        assert_eq!(
            convert_str(-1.0, "kg/h", "kg/s", ConversionContext::default()),
            Err(ConversionError::NegativeValue(Quantity::FlowRate))
        );
    }
}
