//! Density, specific gravity, and API gravity.
//!
//! Specific gravity is a ratio to a reference substance at a stated
//! temperature (usually water), so it is not just another density unit.
//! Converting it to or from an absolute density requires the reference
//! density. °API is defined from specific gravity as 141.5 / SG − 131.5, with
//! SG measured at 60 °F relative to water at 60 °F.

use crate::constants::{
    CUBIC_METRES_PER_CUBIC_FOOT, CUBIC_METRES_PER_US_GALLON, KILOGRAMS_PER_POUND,
};
use crate::context::ConversionContext;
use crate::error::{
    ContextItem, ConversionError, Quantity, validate_finite, validate_non_negative, validate_result,
};
use crate::unit::UnitSet;

const API_NUMERATOR: f64 = 141.5;
const API_OFFSET: f64 = 131.5;
const RELATIVE_REASON: &str = "converting between density and specific gravity or °API";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DensityBasis {
    /// kg/m³.
    Absolute,
    /// Specific gravity relative to the caller's reference density.
    Relative,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Density and relative-density units.
pub enum DensityUnit {
    /// Kilograms per cubic metre (`kg/m³`).
    KilogramsPerCubicMetre,
    /// Grams per cubic centimetre (`g/cm³`).
    GramsPerCubicCentimetre,
    /// Grams per millilitre (`g/mL`).
    GramsPerMillilitre,
    /// Kilograms per litre (`kg/L`).
    KilogramsPerLitre,
    /// Pounds per cubic foot (`lb/ft³`).
    PoundsPerCubicFoot,
    /// Pounds per US gallon (`lb/gal`).
    PoundsPerUsGallon,
    /// Specific gravity (`SG`).
    SpecificGravity,
    /// API gravity (`°API`).
    ApiGravity,
}

impl DensityUnit {
    fn basis(self) -> DensityBasis {
        match self {
            Self::SpecificGravity | Self::ApiGravity => DensityBasis::Relative,
            _ => DensityBasis::Absolute,
        }
    }

    /// Returns the size of one unit in its basis, or `None` for the
    /// non-linear °API scale.
    fn linear_scale(self) -> Option<f64> {
        match self {
            Self::KilogramsPerCubicMetre | Self::SpecificGravity => Some(1.0),
            Self::GramsPerCubicCentimetre | Self::GramsPerMillilitre | Self::KilogramsPerLitre => {
                Some(1_000.0)
            }
            Self::PoundsPerCubicFoot => Some(KILOGRAMS_PER_POUND / CUBIC_METRES_PER_CUBIC_FOOT),
            Self::PoundsPerUsGallon => Some(KILOGRAMS_PER_POUND / CUBIC_METRES_PER_US_GALLON),
            Self::ApiGravity => None,
        }
    }
}

impl UnitSet for DensityUnit {
    const ALL: &'static [Self] = &[
        Self::KilogramsPerCubicMetre,
        Self::GramsPerCubicCentimetre,
        Self::GramsPerMillilitre,
        Self::KilogramsPerLitre,
        Self::PoundsPerCubicFoot,
        Self::PoundsPerUsGallon,
        Self::SpecificGravity,
        Self::ApiGravity,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::KilogramsPerCubicMetre => "kg/m³",
            Self::GramsPerCubicCentimetre => "g/cm³",
            Self::GramsPerMillilitre => "g/mL",
            Self::KilogramsPerLitre => "kg/L",
            Self::PoundsPerCubicFoot => "lb/ft³",
            Self::PoundsPerUsGallon => "lb/gal",
            Self::SpecificGravity => "SG",
            Self::ApiGravity => "°API",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::KilogramsPerCubicMetre => &[],
            Self::GramsPerCubicCentimetre => &["g/cc"],
            Self::GramsPerMillilitre => &[],
            Self::KilogramsPerLitre => &[],
            Self::PoundsPerCubicFoot => &["lbm/ft3"],
            Self::PoundsPerUsGallon => &["lb/usgal", "ppg"],
            Self::SpecificGravity => &["specific-gravity"],
            Self::ApiGravity => &["degapi"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A density stored as kg/m³ or as a specific gravity, depending on its unit.
pub struct Density {
    basis: DensityBasis,
    value: f64,
}

impl Density {
    /// Creates a density from a value and its source unit.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is not finite or is outside the valid range
    /// for `unit`.
    pub fn new(value: f64, unit: DensityUnit) -> Result<Self, ConversionError> {
        let value = match unit.linear_scale() {
            Some(scale) => {
                validate_non_negative(value, Quantity::Density)?;
                value * scale
            }
            None => {
                validate_finite(value)?;
                if value <= -API_OFFSET {
                    return Err(ConversionError::OutOfRange("°API"));
                }
                API_NUMERATOR / (value + API_OFFSET)
            }
        };
        validate_result(value)?;
        Ok(Self {
            basis: unit.basis(),
            value,
        })
    }

    /// Returns this density in `unit`.
    ///
    /// Crossing between absolute density and SG or °API requires
    /// `context.reference_density_kg_m3`.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    pub fn in_unit(
        self,
        unit: DensityUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let value = match (self.basis, unit.basis()) {
            (DensityBasis::Absolute, DensityBasis::Relative) => {
                self.value / context.require(ContextItem::ReferenceDensity, RELATIVE_REASON)?
            }
            (DensityBasis::Relative, DensityBasis::Absolute) => {
                self.value * context.require(ContextItem::ReferenceDensity, RELATIVE_REASON)?
            }
            _ => self.value,
        };
        let value = match unit.linear_scale() {
            Some(scale) => value / scale,
            None if value > 0.0 => API_NUMERATOR / value - API_OFFSET,
            None => return Err(ConversionError::OutOfRange("°API")),
        };
        validate_result(value)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_close, convert_str};

    #[test]
    fn converts_absolute_densities() {
        let none = ConversionContext::default();
        assert_close(convert_str(1.0, "g/cm3", "kg/m3", none).unwrap(), 1_000.0);
        assert_close(
            convert_str(1.0, "g/mL", "lb/ft3", none).unwrap(),
            62.427_960_576_144_6,
        );
        assert_close(
            convert_str(1.0, "kg/L", "lb/gal", none).unwrap(),
            8.345_404_452_194_5,
        );
    }

    #[test]
    fn specific_gravity_needs_a_stated_reference_density() {
        assert_eq!(
            convert_str(0.8, "SG", "kg/m3", ConversionContext::default()),
            Err(ConversionError::MissingContext {
                item: ContextItem::ReferenceDensity,
                reason: RELATIVE_REASON,
            })
        );
        let water = ConversionContext {
            reference_density_kg_m3: Some(999.0),
            ..ConversionContext::default()
        };
        assert_close(convert_str(0.8, "SG", "kg/m3", water).unwrap(), 799.2);
    }

    #[test]
    fn converts_api_gravity() {
        let none = ConversionContext::default();
        assert_close(convert_str(1.0, "SG", "API", none).unwrap(), 10.0);
        assert_close(
            convert_str(35.0, "°API", "SG", none).unwrap(),
            141.5 / 166.5,
        );
        assert_eq!(
            convert_str(-140.0, "API", "SG", none),
            Err(ConversionError::OutOfRange("°API"))
        );
        assert_eq!(
            convert_str(0.0, "SG", "API", none),
            Err(ConversionError::OutOfRange("°API"))
        );
    }
}
