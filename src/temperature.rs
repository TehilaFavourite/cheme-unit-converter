//! Temperature readings and temperature differences.
//!
//! A reading such as 25 °C is a point on a scale, so converting it involves
//! an offset. A difference such as a 10 °C rise only depends on the size of
//! the degree, so 10 Δ°C is 18 Δ°F, not 50 °F. Keeping them as separate
//! quantities stops the reading formula from being applied to a difference.

use crate::error::{ConversionError, validate_finite};
use crate::unit::UnitSet;

const KELVIN_AT_ZERO_CELSIUS: f64 = 273.15;
const KELVIN_PER_RANKINE: f64 = 5.0 / 9.0;
/// Readings this close below 0 K are rounding error from the scale offset
/// (for example −459.67 °F), not a real temperature below absolute zero.
const ABSOLUTE_ZERO_TOLERANCE_KELVIN: f64 = 1e-9;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Scales for temperature readings.
pub enum TemperatureUnit {
    /// Degrees Celsius (`°C`).
    Celsius,
    /// Degrees Fahrenheit (`°F`).
    Fahrenheit,
    /// Kelvin (`K`).
    Kelvin,
    /// Degrees Rankine (`°R`).
    Rankine,
}

impl UnitSet for TemperatureUnit {
    const ALL: &'static [Self] = &[Self::Celsius, Self::Fahrenheit, Self::Kelvin, Self::Rankine];

    fn label(self) -> &'static str {
        match self {
            Self::Celsius => "°C",
            Self::Fahrenheit => "°F",
            Self::Kelvin => "K",
            Self::Rankine => "°R",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Celsius => &["degc", "celsius"],
            Self::Fahrenheit => &["degf", "fahrenheit"],
            Self::Kelvin => &["kelvin"],
            Self::Rankine => &["degr", "rankine"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Degree sizes for temperature differences (intervals).
pub enum TemperatureDifferenceUnit {
    /// Celsius degree difference (`Δ°C`).
    Celsius,
    /// Fahrenheit degree difference (`Δ°F`).
    Fahrenheit,
    /// Kelvin difference (`ΔK`).
    Kelvin,
    /// Rankine degree difference (`Δ°R`).
    Rankine,
}

impl TemperatureDifferenceUnit {
    fn kelvin_per_degree(self) -> f64 {
        match self {
            Self::Celsius | Self::Kelvin => 1.0,
            Self::Fahrenheit | Self::Rankine => KELVIN_PER_RANKINE,
        }
    }
}

impl UnitSet for TemperatureDifferenceUnit {
    const ALL: &'static [Self] = &[Self::Celsius, Self::Fahrenheit, Self::Kelvin, Self::Rankine];

    fn label(self) -> &'static str {
        match self {
            Self::Celsius => "Δ°C",
            Self::Fahrenheit => "Δ°F",
            Self::Kelvin => "ΔK",
            Self::Rankine => "Δ°R",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Celsius => &["dc", "delta-c"],
            Self::Fahrenheit => &["df", "delta-f"],
            Self::Kelvin => &["dk", "delta-k"],
            Self::Rankine => &["dr", "delta-r"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A temperature reading stored canonically as an absolute kelvin value.
pub struct Temperature {
    kelvin: f64,
}

impl Temperature {
    /// Creates a temperature from a value and its source unit.
    ///
    /// Rejects non-finite values and temperatures below absolute zero.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::NonFiniteValue`] or
    /// [`ConversionError::BelowAbsoluteZero`].
    pub fn new(value: f64, unit: TemperatureUnit) -> Result<Self, ConversionError> {
        validate_finite(value)?;
        let kelvin = match unit {
            TemperatureUnit::Celsius => value + KELVIN_AT_ZERO_CELSIUS,
            TemperatureUnit::Fahrenheit => {
                (value - 32.0) * KELVIN_PER_RANKINE + KELVIN_AT_ZERO_CELSIUS
            }
            TemperatureUnit::Kelvin => value,
            TemperatureUnit::Rankine => value * KELVIN_PER_RANKINE,
        };
        if kelvin < -ABSOLUTE_ZERO_TOLERANCE_KELVIN {
            return Err(ConversionError::BelowAbsoluteZero);
        }
        Ok(Self {
            kelvin: kelvin.max(0.0),
        })
    }

    /// Returns the absolute temperature in kelvin.
    pub fn kelvin(self) -> f64 {
        self.kelvin
    }

    /// Returns this temperature expressed in `unit`.
    pub fn in_unit(self, unit: TemperatureUnit) -> f64 {
        match unit {
            TemperatureUnit::Celsius => self.kelvin - KELVIN_AT_ZERO_CELSIUS,
            TemperatureUnit::Fahrenheit => {
                (self.kelvin - KELVIN_AT_ZERO_CELSIUS) / KELVIN_PER_RANKINE + 32.0
            }
            TemperatureUnit::Kelvin => self.kelvin,
            TemperatureUnit::Rankine => self.kelvin / KELVIN_PER_RANKINE,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A temperature difference stored in kelvin. It may be negative.
pub struct TemperatureDifference {
    kelvin: f64,
}

impl TemperatureDifference {
    /// Creates a temperature difference from a value and its source unit.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::NonFiniteValue`] if `value` is not finite.
    pub fn new(value: f64, unit: TemperatureDifferenceUnit) -> Result<Self, ConversionError> {
        validate_finite(value)?;
        Ok(Self {
            kelvin: value * unit.kelvin_per_degree(),
        })
    }

    /// Returns this difference expressed in `unit`.
    pub fn in_unit(self, unit: TemperatureDifferenceUnit) -> f64 {
        self.kelvin / unit.kelvin_per_degree()
    }
}

#[cfg(test)]
mod tests {
    use crate::test_support::{assert_close, convert_str};
    use crate::{ConversionContext, ConversionError};

    fn convert(value: f64, from: &str, to: &str) -> Result<f64, ConversionError> {
        convert_str(value, from, to, ConversionContext::default())
    }

    #[test]
    fn converts_readings_including_rankine() {
        assert_close(convert(100.0, "C", "F").unwrap(), 212.0);
        assert_close(convert(0.0, "F", "R").unwrap(), 459.67);
        assert_close(convert(491.67, "R", "C").unwrap(), 0.0);
        assert_eq!(convert(-459.67, "F", "K"), Ok(0.0));
        assert_eq!(convert(-273.15, "C", "R"), Ok(0.0));
        assert_eq!(
            convert(-459.68, "F", "K"),
            Err(ConversionError::BelowAbsoluteZero)
        );
        assert_eq!(
            convert(-274.0, "C", "K"),
            Err(ConversionError::BelowAbsoluteZero)
        );
    }

    #[test]
    fn converts_differences_without_an_offset() {
        assert_close(convert(10.0, "dC", "dF").unwrap(), 18.0);
        assert_close(convert(-9.0, "ΔF", "ΔK").unwrap(), -5.0);
        assert_close(convert(1.8, "dR", "dC").unwrap(), 1.0);
    }

    #[test]
    fn refuses_to_mix_readings_and_differences() {
        let error = convert(10.0, "C", "dF").unwrap_err();
        assert!(error.to_string().contains("temperature differences"));
    }
}
