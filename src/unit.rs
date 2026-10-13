//! The unit registry: every supported unit, its label, and accepted spellings.

use std::fmt;
use std::str::FromStr;

use crate::composition::ConcentrationUnit;
use crate::density::DensityUnit;
use crate::energy::{EnergyUnit, PowerUnit};
use crate::error::Quantity;
use crate::flow::FlowRateUnit;
use crate::heat_transfer::{
    HeatFluxUnit, HeatTransferCoefficientUnit, SpecificEnthalpyUnit, SpecificHeatUnit,
    ThermalConductivityUnit,
};
use crate::pressure::PressureUnit;
use crate::temperature::{TemperatureDifferenceUnit, TemperatureUnit};
use crate::viscosity::ViscosityUnit;

/// Implemented by each per-quantity unit enum.
pub(crate) trait UnitSet: Copy + 'static {
    const ALL: &'static [Self];

    /// The preferred display label.
    fn label(self) -> &'static str;

    /// Extra spellings accepted by [`parse_unit`] besides the label. Matching
    /// ignores case and the punctuation removed by `normalize`, so aliases
    /// only need to cover genuinely different spellings.
    fn aliases(self) -> &'static [&'static str];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// A unit tagged with the physical quantity to which it belongs.
///
/// `Unit` parses from text with [`parse_unit`] or [`str::parse`], and
/// displays as its preferred label.
///
/// ```
/// use cheme_unit_converter::Unit;
///
/// let unit: Unit = "W/m2K".parse().unwrap();
/// assert_eq!(unit.to_string(), "W/(m²·K)");
/// ```
pub enum Unit {
    /// A temperature reading unit.
    Temperature(TemperatureUnit),
    /// A temperature difference unit.
    TemperatureDifference(TemperatureDifferenceUnit),
    /// An absolute or gauge pressure unit.
    Pressure(PressureUnit),
    /// A mass, volumetric, molar, or standard gas flow unit.
    FlowRate(FlowRateUnit),
    /// A composition or concentration unit.
    Concentration(ConcentrationUnit),
    /// A density, specific gravity, or API gravity unit.
    Density(DensityUnit),
    /// A dynamic or kinematic viscosity unit.
    Viscosity(ViscosityUnit),
    /// An energy unit.
    Energy(EnergyUnit),
    /// A power or heat-duty unit.
    Power(PowerUnit),
    /// A heat transfer coefficient unit.
    HeatTransferCoefficient(HeatTransferCoefficientUnit),
    /// A thermal conductivity unit.
    ThermalConductivity(ThermalConductivityUnit),
    /// A heat flux unit.
    HeatFlux(HeatFluxUnit),
    /// A mass or molar specific heat capacity unit.
    SpecificHeat(SpecificHeatUnit),
    /// A mass or molar specific enthalpy unit.
    SpecificEnthalpy(SpecificEnthalpyUnit),
}

impl fmt::Display for Unit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.label())
    }
}

impl FromStr for Unit {
    type Err = ParseUnitError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        parse_unit(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
/// The error returned when text does not name a supported unit.
pub enum ParseUnitError {
    /// The text matches no supported unit or alias.
    Unrecognized(String),
    /// A milli-prefixed name that case-insensitive matching would read as the
    /// mega-prefixed unit, so it is refused instead.
    AmbiguousPrefix {
        /// The text as given.
        input: String,
        /// The spellings to use instead.
        suggestion: &'static str,
    },
}

impl fmt::Display for ParseUnitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unrecognized(input) => write!(f, "unrecognized unit '{input}'"),
            Self::AmbiguousPrefix { input, suggestion } => {
                write!(f, "'{input}' (milli-) is not supported; use {suggestion}")
            }
        }
    }
}

impl std::error::Error for ParseUnitError {}

impl Unit {
    /// Returns every supported unit, grouped by quantity.
    pub fn all() -> Vec<Self> {
        fn each<U: UnitSet>(wrap: fn(U) -> Unit) -> impl Iterator<Item = Unit> {
            U::ALL.iter().copied().map(wrap)
        }
        each(Self::Temperature)
            .chain(each(Self::TemperatureDifference))
            .chain(each(Self::Pressure))
            .chain(each(Self::FlowRate))
            .chain(each(Self::Concentration))
            .chain(each(Self::Density))
            .chain(each(Self::Viscosity))
            .chain(each(Self::Energy))
            .chain(each(Self::Power))
            .chain(each(Self::HeatTransferCoefficient))
            .chain(each(Self::ThermalConductivity))
            .chain(each(Self::HeatFlux))
            .chain(each(Self::SpecificHeat))
            .chain(each(Self::SpecificEnthalpy))
            .collect()
    }

    /// Returns the quantity this unit measures.
    pub fn quantity(self) -> Quantity {
        self.parts().0
    }

    /// Returns the preferred display label for this unit.
    pub fn label(self) -> &'static str {
        self.parts().1
    }

    /// Returns the extra spellings that [`parse_unit`] accepts besides the label.
    pub fn aliases(self) -> &'static [&'static str] {
        self.parts().2
    }

    fn parts(self) -> (Quantity, &'static str, &'static [&'static str]) {
        match self {
            Self::Temperature(u) => (Quantity::Temperature, u.label(), u.aliases()),
            Self::TemperatureDifference(u) => {
                (Quantity::TemperatureDifference, u.label(), u.aliases())
            }
            Self::Pressure(u) => (Quantity::Pressure, u.label(), u.aliases()),
            Self::FlowRate(u) => (Quantity::FlowRate, u.label(), u.aliases()),
            Self::Concentration(u) => (Quantity::Composition, u.label(), u.aliases()),
            Self::Density(u) => (Quantity::Density, u.label(), u.aliases()),
            Self::Viscosity(u) => (Quantity::Viscosity, u.label(), u.aliases()),
            Self::Energy(u) => (Quantity::Energy, u.label(), u.aliases()),
            Self::Power(u) => (Quantity::Power, u.label(), u.aliases()),
            Self::HeatTransferCoefficient(u) => {
                (Quantity::HeatTransferCoefficient, u.label(), u.aliases())
            }
            Self::ThermalConductivity(u) => (Quantity::ThermalConductivity, u.label(), u.aliases()),
            Self::HeatFlux(u) => (Quantity::HeatFlux, u.label(), u.aliases()),
            Self::SpecificHeat(u) => (Quantity::SpecificHeat, u.label(), u.aliases()),
            Self::SpecificEnthalpy(u) => (Quantity::SpecificEnthalpy, u.label(), u.aliases()),
        }
    }
}

/// Milli-prefixed spellings that case-insensitive matching would otherwise
/// read as mega-prefixed units, a factor of 10⁹ away from what was typed.
const MILLI_MEGA_CONFUSIONS: &[(&str, &str)] = &[
    ("mPa", "'MPa' for megapascals or 'mPa·s' for viscosity"),
    ("mJ", "'MJ' for megajoules"),
    ("mW", "'MW' for megawatts"),
];

/// Parses a unit name or alias.
///
/// Matching ignores case, spaces, `·`, `*`, `.`, `-`, `^`, `°`, and
/// parentheses, reads `²`/`³` as `2`/`3`, and treats `hr` as `h`. So
/// `W/(m²·K)`, `W/m2K`, and `w/m^2.k` are the same unit. [`Unit::all`] and
/// [`Unit::aliases`] list the supported units and their extra spellings.
///
/// # Errors
///
/// Returns [`ParseUnitError::Unrecognized`] if no unit matches, or
/// [`ParseUnitError::AmbiguousPrefix`] for `mPa`, `mJ`, and `mW`.
pub fn parse_unit(value: &str) -> Result<Unit, ParseUnitError> {
    let trimmed = value.trim();
    if let Some((_, suggestion)) = MILLI_MEGA_CONFUSIONS
        .iter()
        .find(|(name, _)| *name == trimmed)
    {
        return Err(ParseUnitError::AmbiguousPrefix {
            input: trimmed.to_string(),
            suggestion,
        });
    }
    let wanted = normalize(trimmed);
    Unit::all()
        .into_iter()
        .find(|unit| spellings(*unit).any(|spelling| spelling == wanted))
        .ok_or_else(|| ParseUnitError::Unrecognized(trimmed.to_string()))
}

/// Returns the normalized label and aliases of `unit`.
fn spellings(unit: Unit) -> impl Iterator<Item = String> {
    std::iter::once(unit.label())
        .chain(unit.aliases().iter().copied())
        .map(normalize)
}

/// Reduces a unit name to a canonical spelling for matching.
fn normalize(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    for character in name.trim().chars() {
        match character {
            '·' | '⋅' | '*' | '.' | '-' | '^' | '°' | '(' | ')' | ' ' => {}
            '²' => normalized.push('2'),
            '³' => normalized.push('3'),
            'μ' => normalized.push('µ'),
            other => normalized.push(other.to_ascii_lowercase()),
        }
    }
    normalized.replace("hr", "h")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn no_spelling_belongs_to_two_units() {
        let mut owners = HashMap::new();
        for unit in Unit::all() {
            for spelling in spellings(unit) {
                if let Some(previous) = owners.insert(spelling.clone(), unit) {
                    assert_eq!(
                        previous, unit,
                        "'{spelling}' is used by both {previous:?} and {unit:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn ignores_punctuation_and_case() {
        let expected = parse_unit("W/(m²·K)").unwrap();
        for spelling in ["W/m2K", "w/m^2.k", "W/(m2 K)", "W/m²/K"] {
            assert_eq!(parse_unit(spelling), Ok(expected), "{spelling}");
        }
        assert_eq!(parse_unit("kg/hr"), parse_unit("kg/h"));
    }

    #[test]
    fn every_label_parses_back_to_its_unit() {
        for unit in Unit::all() {
            assert_eq!(parse_unit(unit.label()), Ok(unit), "label {}", unit.label());
        }
    }

    #[test]
    fn rejects_milli_spellings_that_would_read_as_mega() {
        assert!(parse_unit("mPa").is_err());
        assert!(parse_unit("mJ").is_err());
        assert!(parse_unit("mW").is_err());
        assert_eq!(
            parse_unit("MPa"),
            Ok(Unit::Pressure(PressureUnit::Megapascal))
        );
    }
}
