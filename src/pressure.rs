//! Absolute and gauge pressure.
//!
//! Pressures are stored as absolute pascals. Gauge units (psig, barg, kPag)
//! are measured relative to the local atmosphere, which varies with altitude
//! and weather, so converting between gauge and absolute pressure requires
//! the caller to state the atmospheric pressure.

use crate::constants::{METRES_PER_FOOT, METRES_PER_INCH, STANDARD_GRAVITY};
use crate::context::ConversionContext;
use crate::error::{ContextItem, ConversionError, validate_finite, validate_result};
use crate::unit::UnitSet;

const PASCALS_PER_BAR: f64 = 100_000.0;
const PASCALS_PER_PSI: f64 = 6_894.757_293_168;
const PASCALS_PER_ATMOSPHERE: f64 = 101_325.0;
const PASCALS_PER_TORR: f64 = PASCALS_PER_ATMOSPHERE / 760.0;
/// Conventional millimetre of mercury (mercury at 0 °C under standard gravity).
const PASCALS_PER_MILLIMETRE_MERCURY: f64 = 133.322_387_415;
/// Conventional water column: 1000 kg/m³ under standard gravity.
const PASCALS_PER_METRE_WATER: f64 = 1_000.0 * STANDARD_GRAVITY;
const GAUGE_REASON: &str = "converting between gauge and absolute pressure";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Absolute and gauge pressure units supported by the converter.
pub enum PressureUnit {
    /// Pascal (`Pa`).
    Pascal,
    /// Kilopascal, absolute (`kPa`).
    Kilopascal,
    /// Megapascal (`MPa`).
    Megapascal,
    /// Bar, absolute (`bar`).
    Bar,
    /// Millibar (`mbar`).
    Millibar,
    /// Atmosphere (`atm`).
    Atmosphere,
    /// Pounds per square inch, absolute (`psi`).
    Psi,
    /// Torr (`torr`).
    Torr,
    /// Conventional millimetre of mercury (`mmHg`).
    MillimetresMercury,
    /// Conventional inch of mercury (`inHg`).
    InchesMercury,
    /// Conventional millimetre of water (`mmH₂O`).
    MillimetresWater,
    /// Conventional inch of water (`inH₂O`).
    InchesWater,
    /// Conventional foot of water (`ftH₂O`).
    FeetWater,
    /// Conventional metre of water (`mH₂O`).
    MetresWater,
    /// Kilogram-force per square centimetre (`kgf/cm²`).
    KilogramForcePerSquareCentimetre,
    /// Kilopascal gauge (`kPag`).
    KilopascalGauge,
    /// Bar gauge (`barg`).
    BarGauge,
    /// Pounds per square inch, gauge (`psig`).
    PsiGauge,
}

impl PressureUnit {
    /// Returns whether values in this unit are relative to atmospheric pressure.
    pub fn is_gauge(self) -> bool {
        matches!(
            self,
            Self::KilopascalGauge | Self::BarGauge | Self::PsiGauge
        )
    }

    fn pascals_per_unit(self) -> f64 {
        match self {
            Self::Pascal => 1.0,
            Self::Kilopascal | Self::KilopascalGauge => 1_000.0,
            Self::Megapascal => 1_000_000.0,
            Self::Bar | Self::BarGauge => PASCALS_PER_BAR,
            Self::Millibar => PASCALS_PER_BAR / 1_000.0,
            Self::Atmosphere => PASCALS_PER_ATMOSPHERE,
            Self::Psi | Self::PsiGauge => PASCALS_PER_PSI,
            Self::Torr => PASCALS_PER_TORR,
            Self::MillimetresMercury => PASCALS_PER_MILLIMETRE_MERCURY,
            Self::InchesMercury => PASCALS_PER_MILLIMETRE_MERCURY * METRES_PER_INCH * 1_000.0,
            Self::MillimetresWater => PASCALS_PER_METRE_WATER / 1_000.0,
            Self::InchesWater => PASCALS_PER_METRE_WATER * METRES_PER_INCH,
            Self::FeetWater => PASCALS_PER_METRE_WATER * METRES_PER_FOOT,
            Self::MetresWater => PASCALS_PER_METRE_WATER,
            Self::KilogramForcePerSquareCentimetre => STANDARD_GRAVITY * 10_000.0,
        }
    }
}

impl UnitSet for PressureUnit {
    const ALL: &'static [Self] = &[
        Self::Pascal,
        Self::Kilopascal,
        Self::Megapascal,
        Self::Bar,
        Self::Millibar,
        Self::Atmosphere,
        Self::Psi,
        Self::Torr,
        Self::MillimetresMercury,
        Self::InchesMercury,
        Self::MillimetresWater,
        Self::InchesWater,
        Self::FeetWater,
        Self::MetresWater,
        Self::KilogramForcePerSquareCentimetre,
        Self::KilopascalGauge,
        Self::BarGauge,
        Self::PsiGauge,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Pascal => "Pa",
            Self::Kilopascal => "kPa",
            Self::Megapascal => "MPa",
            Self::Bar => "bar",
            Self::Millibar => "mbar",
            Self::Atmosphere => "atm",
            Self::Psi => "psi",
            Self::Torr => "torr",
            Self::MillimetresMercury => "mmHg",
            Self::InchesMercury => "inHg",
            Self::MillimetresWater => "mmH₂O",
            Self::InchesWater => "inH₂O",
            Self::FeetWater => "ftH₂O",
            Self::MetresWater => "mH₂O",
            Self::KilogramForcePerSquareCentimetre => "kgf/cm²",
            Self::KilopascalGauge => "kPag",
            Self::BarGauge => "barg",
            Self::PsiGauge => "psig",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Pascal => &["pascal"],
            Self::Kilopascal => &["kpaa"],
            Self::Megapascal => &[],
            Self::Bar => &["bara"],
            Self::Millibar => &[],
            Self::Atmosphere => &[],
            Self::Psi => &["psia"],
            Self::Torr => &[],
            Self::MillimetresMercury => &[],
            Self::InchesMercury => &[],
            Self::MillimetresWater => &["mmh2o", "mmwc"],
            Self::InchesWater => &["inh2o", "inwc"],
            Self::FeetWater => &["fth2o", "ftwc"],
            Self::MetresWater => &["mh2o", "mwc"],
            Self::KilogramForcePerSquareCentimetre => &[],
            Self::KilopascalGauge => &[],
            Self::BarGauge => &[],
            Self::PsiGauge => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A non-negative absolute pressure stored in pascals.
pub struct Pressure {
    pascals: f64,
}

impl Pressure {
    /// Creates an absolute pressure from a value and its source unit.
    ///
    /// Gauge units need `context.atmospheric_pressure`. The resulting
    /// absolute pressure must not be negative.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is not finite or is outside the valid range
    /// for `unit`.
    /// Gauge units also return [`ConversionError::MissingContext`] without
    /// `context.atmospheric_pressure`.
    pub fn new(
        value: f64,
        unit: PressureUnit,
        context: &ConversionContext,
    ) -> Result<Self, ConversionError> {
        validate_finite(value)?;
        let mut pascals = value * unit.pascals_per_unit();
        if unit.is_gauge() {
            pascals += context.require(ContextItem::AtmosphericPressure, GAUGE_REASON)?;
        }
        validate_result(pascals)?;
        if pascals < 0.0 {
            return Err(ConversionError::NegativePressure);
        }
        Ok(Self { pascals })
    }

    /// Returns the absolute pressure in pascals.
    pub fn pascals(self) -> f64 {
        self.pascals
    }

    /// Returns this pressure expressed in `unit`.
    ///
    /// Gauge units need `context.atmospheric_pressure`.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    pub fn in_unit(
        self,
        unit: PressureUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let mut pascals = self.pascals;
        if unit.is_gauge() {
            pascals -= context.require(ContextItem::AtmosphericPressure, GAUGE_REASON)?;
        }
        let value = pascals / unit.pascals_per_unit();
        validate_result(value)?;
        Ok(value)
    }
}

/// Converts a pressure, allowing gauge-to-gauge conversion without an
/// atmospheric pressure because it only rescales the pressure difference.
pub(crate) fn convert_pressure(
    value: f64,
    from: PressureUnit,
    to: PressureUnit,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    if from.is_gauge() && to.is_gauge() && context.atmospheric_pressure.is_none() {
        validate_finite(value)?;
        let result = value * from.pascals_per_unit() / to.pascals_per_unit();
        validate_result(result)?;
        return Ok(result);
    }
    Pressure::new(value, from, context)?.in_unit(to, context)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::ContextItem;
    use crate::test_support::{assert_close, convert_str};

    fn with_atmosphere(value: f64, unit: PressureUnit) -> ConversionContext {
        let atmosphere = Pressure::new(value, unit, &ConversionContext::default()).unwrap();
        ConversionContext {
            atmospheric_pressure: Some(atmosphere),
            ..ConversionContext::default()
        }
    }

    #[test]
    fn converts_absolute_units() {
        let none = ConversionContext::default();
        assert_close(convert_str(1.0, "atm", "kPa", none).unwrap(), 101.325);
        assert_close(convert_str(760.0, "torr", "atm", none).unwrap(), 1.0);
        assert_close(convert_str(1.0, "inHg", "mmHg", none).unwrap(), 25.4);
        assert_close(convert_str(10.0, "mH2O", "kPa", none).unwrap(), 98.0665);
        assert_close(convert_str(1.0, "MPa", "bar", none).unwrap(), 10.0);
    }

    #[test]
    fn gauge_conversion_needs_stated_atmospheric_pressure() {
        assert_eq!(
            convert_str(30.0, "psig", "psia", ConversionContext::default()),
            Err(ConversionError::MissingContext {
                item: ContextItem::AtmosphericPressure,
                reason: GAUGE_REASON,
            })
        );
        let context = with_atmosphere(14.7, PressureUnit::Psi);
        assert_close(convert_str(30.0, "psig", "psia", context).unwrap(), 44.7);
        assert_close(convert_str(14.7, "psia", "psig", context).unwrap(), 0.0);
    }

    #[test]
    fn gauge_to_gauge_rescales_without_atmospheric_pressure() {
        let result = convert_str(1.0, "barg", "psig", ConversionContext::default()).unwrap();
        assert_close(result, PASCALS_PER_BAR / PASCALS_PER_PSI);
    }

    #[test]
    fn allows_vacuum_gauge_but_not_negative_absolute_pressure() {
        let context = with_atmosphere(101.325, PressureUnit::Kilopascal);
        assert_close(convert_str(-50.0, "kPag", "kPa", context).unwrap(), 51.325);
        assert_eq!(
            convert_str(-200.0, "kPag", "kPa", context),
            Err(ConversionError::NegativePressure)
        );
        assert_eq!(
            convert_str(-1.0, "bar", "psi", ConversionContext::default()),
            Err(ConversionError::NegativePressure)
        );
    }
}
