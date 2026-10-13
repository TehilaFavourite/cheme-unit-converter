//! Dynamic and kinematic viscosity.
//!
//! Dynamic viscosity (Pa·s, cP) and kinematic viscosity (m²/s, cSt) are
//! different quantities related by ν = μ / ρ, so converting between them
//! requires the fluid density.

use crate::constants::{KILOGRAMS_PER_POUND, METRES_PER_FOOT, SECONDS_PER_HOUR};
use crate::context::ConversionContext;
use crate::error::{
    ContextItem, ConversionError, Quantity, validate_non_negative, validate_result,
};
use crate::unit::UnitSet;

const DYNAMIC_KINEMATIC: &str = "converting between dynamic and kinematic viscosity";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ViscosityBasis {
    /// Pa·s.
    Dynamic,
    /// m²/s.
    Kinematic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Dynamic and kinematic viscosity units.
pub enum ViscosityUnit {
    /// Pascal second (`Pa·s`).
    PascalSecond,
    /// Millipascal second (`mPa·s`).
    MillipascalSecond,
    /// Centipoise (`cP`).
    Centipoise,
    /// Poise (`P`).
    Poise,
    /// Pounds per foot second (`lb/(ft·s)`).
    PoundsPerFootSecond,
    /// Pounds per foot hour (`lb/(ft·h)`).
    PoundsPerFootHour,
    /// Square metres per second (`m²/s`).
    SquareMetresPerSecond,
    /// Square millimetres per second (`mm²/s`).
    SquareMillimetresPerSecond,
    /// Centistokes (`cSt`).
    Centistokes,
    /// Stokes (`St`).
    Stokes,
    /// Square feet per second (`ft²/s`).
    SquareFeetPerSecond,
}

impl ViscosityUnit {
    fn basis_and_scale(self) -> (ViscosityBasis, f64) {
        use ViscosityBasis::{Dynamic, Kinematic};
        match self {
            Self::PascalSecond => (Dynamic, 1.0),
            Self::MillipascalSecond | Self::Centipoise => (Dynamic, 1e-3),
            Self::Poise => (Dynamic, 0.1),
            Self::PoundsPerFootSecond => (Dynamic, KILOGRAMS_PER_POUND / METRES_PER_FOOT),
            Self::PoundsPerFootHour => (
                Dynamic,
                KILOGRAMS_PER_POUND / METRES_PER_FOOT / SECONDS_PER_HOUR,
            ),
            Self::SquareMetresPerSecond => (Kinematic, 1.0),
            Self::SquareMillimetresPerSecond | Self::Centistokes => (Kinematic, 1e-6),
            Self::Stokes => (Kinematic, 1e-4),
            Self::SquareFeetPerSecond => (Kinematic, METRES_PER_FOOT * METRES_PER_FOOT),
        }
    }
}

impl UnitSet for ViscosityUnit {
    const ALL: &'static [Self] = &[
        Self::PascalSecond,
        Self::MillipascalSecond,
        Self::Centipoise,
        Self::Poise,
        Self::PoundsPerFootSecond,
        Self::PoundsPerFootHour,
        Self::SquareMetresPerSecond,
        Self::SquareMillimetresPerSecond,
        Self::Centistokes,
        Self::Stokes,
        Self::SquareFeetPerSecond,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::PascalSecond => "Pa·s",
            Self::MillipascalSecond => "mPa·s",
            Self::Centipoise => "cP",
            Self::Poise => "P",
            Self::PoundsPerFootSecond => "lb/(ft·s)",
            Self::PoundsPerFootHour => "lb/(ft·h)",
            Self::SquareMetresPerSecond => "m²/s",
            Self::SquareMillimetresPerSecond => "mm²/s",
            Self::Centistokes => "cSt",
            Self::Stokes => "St",
            Self::SquareFeetPerSecond => "ft²/s",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::PascalSecond => &[],
            Self::MillipascalSecond => &[],
            Self::Centipoise => &["centipoise"],
            Self::Poise => &["poise"],
            Self::PoundsPerFootSecond => &["lb/ft/s"],
            Self::PoundsPerFootHour => &["lb/ft/h"],
            Self::SquareMetresPerSecond => &[],
            Self::SquareMillimetresPerSecond => &[],
            Self::Centistokes => &["centistokes"],
            Self::Stokes => &["stokes"],
            Self::SquareFeetPerSecond => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A non-negative viscosity stored in the SI unit of its basis.
pub struct Viscosity {
    basis: ViscosityBasis,
    value: f64,
}

impl Viscosity {
    /// Creates a viscosity from a value and its source unit.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is not finite or is outside the valid range
    /// for `unit`.
    pub fn new(value: f64, unit: ViscosityUnit) -> Result<Self, ConversionError> {
        validate_non_negative(value, Quantity::Viscosity)?;
        let (basis, scale) = unit.basis_and_scale();
        let value = value * scale;
        validate_result(value)?;
        Ok(Self { basis, value })
    }

    /// Returns this viscosity in `unit`.
    ///
    /// Converting between dynamic and kinematic viscosity requires
    /// `context.density_kg_m3`.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    pub fn in_unit(
        self,
        unit: ViscosityUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let (basis, scale) = unit.basis_and_scale();
        let value = match (self.basis, basis) {
            (ViscosityBasis::Dynamic, ViscosityBasis::Kinematic) => {
                self.value / context.require(ContextItem::Density, DYNAMIC_KINEMATIC)?
            }
            (ViscosityBasis::Kinematic, ViscosityBasis::Dynamic) => {
                self.value * context.require(ContextItem::Density, DYNAMIC_KINEMATIC)?
            }
            _ => self.value,
        } / scale;
        validate_result(value)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_close, convert_str};

    #[test]
    fn converts_within_dynamic_and_kinematic_viscosity() {
        let none = ConversionContext::default();
        assert_close(convert_str(1.0, "cP", "Pa.s", none).unwrap(), 1e-3);
        assert_close(convert_str(1.0, "P", "mPa·s", none).unwrap(), 100.0);
        assert_close(
            convert_str(1.0, "lb/ft/s", "cP", none).unwrap(),
            1_488.163_943_569_553_6,
        );
        assert_close(convert_str(1.0, "St", "cSt", none).unwrap(), 100.0);
    }

    #[test]
    fn dynamic_to_kinematic_needs_density() {
        assert_eq!(
            convert_str(1.0, "cP", "cSt", ConversionContext::default()),
            Err(ConversionError::MissingContext {
                item: ContextItem::Density,
                reason: DYNAMIC_KINEMATIC,
            })
        );
        let oil = ConversionContext {
            density_kg_m3: Some(850.0),
            ..ConversionContext::default()
        };
        assert_close(convert_str(42.5, "cP", "cSt", oil).unwrap(), 50.0);
    }
}
