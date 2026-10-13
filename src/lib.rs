//! Unit-safe conversions for common chemical engineering measurements.
//!
//! Supported quantities are temperature, temperature difference, pressure
//! (absolute and gauge), flow rate (mass, volumetric, molar, and standard
//! gas), composition, density (including specific gravity and °API),
//! viscosity (dynamic and kinematic), energy, power, and the heat-transfer
//! properties: heat transfer coefficient, thermal conductivity, heat flux,
//! specific heat capacity, and specific enthalpy. Each quantity has its own newtype, so a
//! [`Temperature`] cannot be passed where a [`Pressure`] is expected, and a
//! temperature reading cannot be converted as if it were a difference.
//!
//! # Examples
//!
//! ```
//! use cheme_unit_converter::{ConversionContext, convert, parse_unit};
//!
//! let celsius = parse_unit("C").unwrap();
//! let fahrenheit = parse_unit("F").unwrap();
//! let result = convert(25.0, celsius, fahrenheit, ConversionContext::default()).unwrap();
//! assert!((result - 77.0).abs() < 1e-10);
//! ```
//!
//! # Context is never assumed
//!
//! Some conversions depend on a material property, a reference condition, or
//! a convention: mass to volume flow needs density, gauge pressure needs the
//! local atmospheric pressure, and standard gas volume needs the reference
//! temperature and pressure that define "standard". The caller supplies these
//! in [`ConversionContext`]. When one is missing, the conversion fails with
//! [`ConversionError::MissingContext`], which names the missing item and the
//! step that needs it, rather than silently using a typical value.

#![warn(missing_docs)]

mod composition;
mod constants;
mod context;
mod density;
mod energy;
mod error;
mod flow;
mod heat_transfer;
mod linear;
mod pressure;
mod temperature;
mod unit;
mod viscosity;

pub use composition::{Concentration, ConcentrationUnit};
pub use context::ConversionContext;
pub use density::{Density, DensityUnit};
pub use energy::{Energy, EnergyUnit, Power, PowerUnit};
pub use error::{ContextItem, ConversionError, Quantity};
pub use flow::{FlowRate, FlowRateUnit};
pub use heat_transfer::{
    HeatFlux, HeatFluxUnit, HeatTransferCoefficient, HeatTransferCoefficientUnit, SpecificEnthalpy,
    SpecificEnthalpyUnit, SpecificHeat, SpecificHeatUnit, ThermalConductivity,
    ThermalConductivityUnit,
};
pub use pressure::{Pressure, PressureUnit};
pub use temperature::{
    Temperature, TemperatureDifference, TemperatureDifferenceUnit, TemperatureUnit,
};
pub use unit::{ParseUnitError, Unit, parse_unit};
pub use viscosity::{Viscosity, ViscosityUnit};

use error::validate_result;

/// Converts `value` between two units of the same physical quantity.
///
/// [`ConversionContext`] supplies any properties or reference conditions
/// needed to cross between bases, such as mass and volumetric flow. The
/// result is rejected if the units describe different quantities, the input
/// is invalid, or required context is missing.
///
/// # Examples
///
/// ```
/// use cheme_unit_converter::{ConversionContext, convert, parse_unit};
///
/// let result = convert(
///     1.0,
///     parse_unit("atm").unwrap(),
///     parse_unit("kPa").unwrap(),
///     ConversionContext::default(),
/// ).unwrap();
/// assert!((result - 101.325).abs() < 1e-10);
/// ```
///
/// # Errors
///
/// Returns [`ConversionError::IncompatibleUnits`] if the units measure
/// different quantities. Otherwise returns the errors of the quantity's own
/// `new` and `in_unit`: invalid input, missing or invalid context, or overflow.
pub fn convert(
    value: f64,
    from: Unit,
    to: Unit,
    context: ConversionContext,
) -> Result<f64, ConversionError> {
    match (from, to) {
        (Unit::Temperature(from), Unit::Temperature(to)) => {
            let result = Temperature::new(value, from)?.in_unit(to);
            validate_result(result)?;
            Ok(result)
        }
        (Unit::TemperatureDifference(from), Unit::TemperatureDifference(to)) => {
            let result = TemperatureDifference::new(value, from)?.in_unit(to);
            validate_result(result)?;
            Ok(result)
        }
        (Unit::Pressure(from), Unit::Pressure(to)) => {
            pressure::convert_pressure(value, from, to, &context)
        }
        (Unit::FlowRate(from), Unit::FlowRate(to)) => {
            FlowRate::new(value, from)?.in_unit(to, &context)
        }
        (Unit::Concentration(from), Unit::Concentration(to)) => {
            Concentration::new(value, from)?.in_unit(to, &context)
        }
        (Unit::Density(from), Unit::Density(to)) => {
            Density::new(value, from)?.in_unit(to, &context)
        }
        (Unit::Viscosity(from), Unit::Viscosity(to)) => {
            Viscosity::new(value, from)?.in_unit(to, &context)
        }
        (Unit::Energy(from), Unit::Energy(to)) => Energy::new(value, from)?.in_unit(to),
        (Unit::Power(from), Unit::Power(to)) => Power::new(value, from)?.in_unit(to),
        (Unit::HeatTransferCoefficient(from), Unit::HeatTransferCoefficient(to)) => {
            HeatTransferCoefficient::new(value, from)?.in_unit(to)
        }
        (Unit::ThermalConductivity(from), Unit::ThermalConductivity(to)) => {
            ThermalConductivity::new(value, from)?.in_unit(to)
        }
        (Unit::HeatFlux(from), Unit::HeatFlux(to)) => HeatFlux::new(value, from)?.in_unit(to),
        (Unit::SpecificHeat(from), Unit::SpecificHeat(to)) => {
            SpecificHeat::new(value, from)?.in_unit(to, &context)
        }
        (Unit::SpecificEnthalpy(from), Unit::SpecificEnthalpy(to)) => {
            SpecificEnthalpy::new(value, from)?.in_unit(to, &context)
        }
        _ => Err(ConversionError::IncompatibleUnits {
            from: from.quantity(),
            to: to.quantity(),
        }),
    }
}

#[cfg(test)]
mod test_support {
    use crate::{ConversionContext, ConversionError, convert, parse_unit};

    /// Converts using unit names, panicking if a name is not recognized.
    pub(crate) fn convert_str(
        value: f64,
        from: &str,
        to: &str,
        context: ConversionContext,
    ) -> Result<f64, ConversionError> {
        let from = parse_unit(from).expect("test unit should be supported");
        let to = parse_unit(to).expect("test unit should be supported");
        convert(value, from, to, context)
    }

    /// Asserts agreement to about nine significant figures.
    pub(crate) fn assert_close(actual: f64, expected: f64) {
        let tolerance = 1e-9 * expected.abs().max(1.0);
        assert!(
            (actual - expected).abs() <= tolerance,
            "expected {expected}, got {actual}"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::convert_str;

    #[test]
    fn rejects_incompatible_units_and_non_finite_values() {
        assert_eq!(
            convert_str(1.0, "bar", "kg/h", ConversionContext::default()),
            Err(ConversionError::IncompatibleUnits {
                from: Quantity::Pressure,
                to: Quantity::FlowRate,
            })
        );
        assert_eq!(
            convert_str(f64::NAN, "bar", "psi", ConversionContext::default()),
            Err(ConversionError::NonFiniteValue)
        );
        assert_eq!(
            convert_str(f64::INFINITY, "dC", "dF", ConversionContext::default()),
            Err(ConversionError::NonFiniteValue)
        );
    }

    #[test]
    fn missing_context_message_names_the_property_and_the_reason() {
        let error = convert_str(1.0, "cP", "cSt", ConversionContext::default()).unwrap_err();
        assert_eq!(
            error.to_string(),
            "converting between dynamic and kinematic viscosity requires the fluid density (kg/m³)"
        );
    }
}
