//! Heat-transfer properties: heat transfer coefficient, thermal conductivity,
//! heat flux, specific heat capacity, and specific enthalpy.
//!
//! The "per degree" in these units is a temperature difference, so °F and °R
//! degrees are 5/9 K and °C degrees are 1 K, with no offset. That is why
//! 1 BTU/(lb·°F) is exactly 4.1868 kJ/(kg·K).
//!
//! Specific heat and enthalpy come on a mass basis (per kg, per lb) and a
//! molar basis (per mol, per lbmol). Crossing between them needs the molar
//! mass.
//!
//! Calorie conventions follow the references each group comes from. The
//! transport units (heat transfer coefficient, conductivity, heat flux) use
//! the International Table calorie (4.1868 J), as engineering handbooks do,
//! so 1 kcal/(h·m²·°C) is 1.163 W/(m²·K). Specific heat and enthalpy use the
//! thermochemical calorie (4.184 J), as chemistry data does. The two differ by
//! about 0.07%.

use crate::constants::{
    GRAMS_PER_KILOGRAM, JOULES_PER_BTU, JOULES_PER_CALORIE, JOULES_PER_IT_CALORIE,
    KELVIN_PER_FAHRENHEIT_DEGREE, KILOGRAMS_PER_POUND, METRES_PER_FOOT, MOLES_PER_POUND_MOLE,
    SECONDS_PER_HOUR, SQUARE_METRES_PER_SQUARE_FOOT,
};
use crate::context::ConversionContext;
use crate::error::{
    ContextItem, ConversionError, Quantity, Sign, validate_finite, validate_result, validate_sign,
};
use crate::linear::{LinearUnit, linear_quantity};
use crate::unit::UnitSet;

/// Thermochemical kilocalorie, used by the specific heat and enthalpy units.
const KILOCALORIE: f64 = 1e3 * JOULES_PER_CALORIE;
/// International Table kilocalorie, used by the transport-property units.
const IT_KILOCALORIE: f64 = 1e3 * JOULES_PER_IT_CALORIE;
const SQUARE_METRES_PER_SQUARE_CENTIMETRE: f64 = 1e-4;
const METRES_PER_CENTIMETRE: f64 = 1e-2;
const SPECIFIC_HEAT_REASON: &str = "converting between mass and molar specific heat capacity";
const ENTHALPY_REASON: &str = "converting between mass and molar specific enthalpy";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Heat transfer coefficient (film coefficient, overall U) units.
pub enum HeatTransferCoefficientUnit {
    /// Watts per square metre kelvin (`W/(m²·K)`).
    WattsPerSquareMetreKelvin,
    /// Kilowatts per square metre kelvin (`kW/(m²·K)`).
    KilowattsPerSquareMetreKelvin,
    /// BTU per hour square foot degree Fahrenheit (`BTU/(h·ft²·°F)`).
    BtuPerHourSquareFootFahrenheit,
    /// International Table kilocalories per hour square metre degree Celsius (`kcal/(h·m²·°C)`).
    KilocaloriesPerHourSquareMetreCelsius,
    /// International Table calories per second square centimetre degree Celsius (`cal/(s·cm²·°C)`).
    CaloriesPerSecondSquareCentimetreCelsius,
}

impl LinearUnit for HeatTransferCoefficientUnit {
    fn si_per_unit(self) -> f64 {
        match self {
            Self::WattsPerSquareMetreKelvin => 1.0,
            Self::KilowattsPerSquareMetreKelvin => 1e3,
            Self::BtuPerHourSquareFootFahrenheit => {
                JOULES_PER_BTU
                    / (SECONDS_PER_HOUR
                        * SQUARE_METRES_PER_SQUARE_FOOT
                        * KELVIN_PER_FAHRENHEIT_DEGREE)
            }
            Self::KilocaloriesPerHourSquareMetreCelsius => IT_KILOCALORIE / SECONDS_PER_HOUR,
            Self::CaloriesPerSecondSquareCentimetreCelsius => {
                JOULES_PER_IT_CALORIE / SQUARE_METRES_PER_SQUARE_CENTIMETRE
            }
        }
    }
}

impl UnitSet for HeatTransferCoefficientUnit {
    const ALL: &'static [Self] = &[
        Self::WattsPerSquareMetreKelvin,
        Self::KilowattsPerSquareMetreKelvin,
        Self::BtuPerHourSquareFootFahrenheit,
        Self::KilocaloriesPerHourSquareMetreCelsius,
        Self::CaloriesPerSecondSquareCentimetreCelsius,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::WattsPerSquareMetreKelvin => "W/(m²·K)",
            Self::KilowattsPerSquareMetreKelvin => "kW/(m²·K)",
            Self::BtuPerHourSquareFootFahrenheit => "BTU/(h·ft²·°F)",
            Self::KilocaloriesPerHourSquareMetreCelsius => "kcal/(h·m²·°C)",
            Self::CaloriesPerSecondSquareCentimetreCelsius => "cal/(s·cm²·°C)",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::WattsPerSquareMetreKelvin => &["w/m2/k", "w/(m²·°c)", "w/m2/c"],
            Self::KilowattsPerSquareMetreKelvin => &["kw/m2/k", "kw/(m²·°c)", "kw/m2/c"],
            Self::BtuPerHourSquareFootFahrenheit => &["btu/h/ft2/f", "btu/(ft²·h·°f)"],
            Self::KilocaloriesPerHourSquareMetreCelsius => &["kcal/h/m2/c", "kcal/(m²·h·°c)"],
            Self::CaloriesPerSecondSquareCentimetreCelsius => &["cal/s/cm2/c"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Thermal conductivity units.
pub enum ThermalConductivityUnit {
    /// Watts per metre kelvin (`W/(m·K)`).
    WattsPerMetreKelvin,
    /// BTU per hour foot degree Fahrenheit (`BTU/(h·ft·°F)`).
    BtuPerHourFootFahrenheit,
    /// International Table kilocalories per hour metre degree Celsius (`kcal/(h·m·°C)`).
    KilocaloriesPerHourMetreCelsius,
    /// International Table calories per second centimetre degree Celsius (`cal/(s·cm·°C)`).
    CaloriesPerSecondCentimetreCelsius,
}

impl LinearUnit for ThermalConductivityUnit {
    fn si_per_unit(self) -> f64 {
        match self {
            Self::WattsPerMetreKelvin => 1.0,
            Self::BtuPerHourFootFahrenheit => {
                JOULES_PER_BTU / (SECONDS_PER_HOUR * METRES_PER_FOOT * KELVIN_PER_FAHRENHEIT_DEGREE)
            }
            Self::KilocaloriesPerHourMetreCelsius => IT_KILOCALORIE / SECONDS_PER_HOUR,
            Self::CaloriesPerSecondCentimetreCelsius => {
                JOULES_PER_IT_CALORIE / METRES_PER_CENTIMETRE
            }
        }
    }
}

impl UnitSet for ThermalConductivityUnit {
    const ALL: &'static [Self] = &[
        Self::WattsPerMetreKelvin,
        Self::BtuPerHourFootFahrenheit,
        Self::KilocaloriesPerHourMetreCelsius,
        Self::CaloriesPerSecondCentimetreCelsius,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::WattsPerMetreKelvin => "W/(m·K)",
            Self::BtuPerHourFootFahrenheit => "BTU/(h·ft·°F)",
            Self::KilocaloriesPerHourMetreCelsius => "kcal/(h·m·°C)",
            Self::CaloriesPerSecondCentimetreCelsius => "cal/(s·cm·°C)",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::WattsPerMetreKelvin => &["w/m/k", "w/(m·°c)", "w/m/c"],
            Self::BtuPerHourFootFahrenheit => &["btu/h/ft/f", "btu/(ft·h·°f)"],
            Self::KilocaloriesPerHourMetreCelsius => &["kcal/h/m/c", "kcal/(m·h·°c)"],
            Self::CaloriesPerSecondCentimetreCelsius => &["cal/s/cm/c"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Heat flux units.
pub enum HeatFluxUnit {
    /// Watts per square metre (`W/m²`).
    WattsPerSquareMetre,
    /// Kilowatts per square metre (`kW/m²`).
    KilowattsPerSquareMetre,
    /// BTU per hour square foot (`BTU/(h·ft²)`).
    BtuPerHourSquareFoot,
    /// International Table kilocalories per hour square metre (`kcal/(h·m²)`).
    KilocaloriesPerHourSquareMetre,
}

impl LinearUnit for HeatFluxUnit {
    fn si_per_unit(self) -> f64 {
        match self {
            Self::WattsPerSquareMetre => 1.0,
            Self::KilowattsPerSquareMetre => 1e3,
            Self::BtuPerHourSquareFoot => {
                JOULES_PER_BTU / (SECONDS_PER_HOUR * SQUARE_METRES_PER_SQUARE_FOOT)
            }
            Self::KilocaloriesPerHourSquareMetre => IT_KILOCALORIE / SECONDS_PER_HOUR,
        }
    }
}

impl UnitSet for HeatFluxUnit {
    const ALL: &'static [Self] = &[
        Self::WattsPerSquareMetre,
        Self::KilowattsPerSquareMetre,
        Self::BtuPerHourSquareFoot,
        Self::KilocaloriesPerHourSquareMetre,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::WattsPerSquareMetre => "W/m²",
            Self::KilowattsPerSquareMetre => "kW/m²",
            Self::BtuPerHourSquareFoot => "BTU/(h·ft²)",
            Self::KilocaloriesPerHourSquareMetre => "kcal/(h·m²)",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::WattsPerSquareMetre => &[],
            Self::KilowattsPerSquareMetre => &[],
            Self::BtuPerHourSquareFoot => &["btu/h/ft2", "btu/(ft²·h)"],
            Self::KilocaloriesPerHourSquareMetre => &["kcal/h/m2", "kcal/(m²·h)"],
        }
    }
}

linear_quantity! {
    /// A heat transfer coefficient stored in W/(m²·K).
    HeatTransferCoefficient(
        HeatTransferCoefficientUnit,
        Quantity::HeatTransferCoefficient,
        Sign::NonNegative
    )
}

linear_quantity! {
    /// A thermal conductivity stored in W/(m·K).
    ThermalConductivity(ThermalConductivityUnit, Quantity::ThermalConductivity, Sign::NonNegative)
}

linear_quantity! {
    /// A heat flux stored in W/m². Its sign gives the direction of heat flow.
    HeatFlux(HeatFluxUnit, Quantity::HeatFlux, Sign::Any)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AmountBasis {
    /// Per kilogram.
    Mass,
    /// Per mole.
    Molar,
}

/// Converts a per-kilogram value to per-mole, or back, using the molar mass.
fn rebase(
    value: f64,
    from: AmountBasis,
    to: AmountBasis,
    context: &ConversionContext,
    reason: &'static str,
) -> Result<f64, ConversionError> {
    Ok(match (from, to) {
        (AmountBasis::Mass, AmountBasis::Molar) => {
            value * context.require(ContextItem::MolarMass, reason)? / GRAMS_PER_KILOGRAM
        }
        (AmountBasis::Molar, AmountBasis::Mass) => {
            value * GRAMS_PER_KILOGRAM / context.require(ContextItem::MolarMass, reason)?
        }
        _ => value,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Specific heat capacity units, on a mass or molar basis.
pub enum SpecificHeatUnit {
    /// Joules per kilogram kelvin (`J/(kg·K)`).
    JoulesPerKilogramKelvin,
    /// Kilojoules per kilogram kelvin (`kJ/(kg·K)`).
    KilojoulesPerKilogramKelvin,
    /// Joules per gram kelvin (`J/(g·K)`).
    JoulesPerGramKelvin,
    /// BTU per pound degree Fahrenheit (`BTU/(lb·°F)`).
    BtuPerPoundFahrenheit,
    /// Calories per gram degree Celsius (`cal/(g·°C)`).
    CaloriesPerGramCelsius,
    /// Kilocalories per kilogram degree Celsius (`kcal/(kg·°C)`).
    KilocaloriesPerKilogramCelsius,
    /// Joules per mole kelvin (`J/(mol·K)`).
    JoulesPerMoleKelvin,
    /// Kilojoules per kilomole kelvin (`kJ/(kmol·K)`).
    KilojoulesPerKilomoleKelvin,
    /// BTU per pound mole degree Fahrenheit (`BTU/(lbmol·°F)`).
    BtuPerPoundMoleFahrenheit,
    /// Calories per mole kelvin (`cal/(mol·K)`).
    CaloriesPerMoleKelvin,
}

impl SpecificHeatUnit {
    /// Returns the basis and the size of one unit in J/(kg·K) or J/(mol·K).
    fn basis_and_scale(self) -> (AmountBasis, f64) {
        use AmountBasis::{Mass, Molar};
        let per_fahrenheit = 1.0 / KELVIN_PER_FAHRENHEIT_DEGREE;
        match self {
            Self::JoulesPerKilogramKelvin => (Mass, 1.0),
            Self::KilojoulesPerKilogramKelvin | Self::JoulesPerGramKelvin => (Mass, 1e3),
            Self::BtuPerPoundFahrenheit => {
                (Mass, JOULES_PER_BTU / KILOGRAMS_PER_POUND * per_fahrenheit)
            }
            Self::CaloriesPerGramCelsius | Self::KilocaloriesPerKilogramCelsius => {
                (Mass, KILOCALORIE)
            }
            Self::JoulesPerMoleKelvin | Self::KilojoulesPerKilomoleKelvin => (Molar, 1.0),
            Self::BtuPerPoundMoleFahrenheit => (
                Molar,
                JOULES_PER_BTU / MOLES_PER_POUND_MOLE * per_fahrenheit,
            ),
            Self::CaloriesPerMoleKelvin => (Molar, JOULES_PER_CALORIE),
        }
    }
}

impl UnitSet for SpecificHeatUnit {
    const ALL: &'static [Self] = &[
        Self::JoulesPerKilogramKelvin,
        Self::KilojoulesPerKilogramKelvin,
        Self::JoulesPerGramKelvin,
        Self::BtuPerPoundFahrenheit,
        Self::CaloriesPerGramCelsius,
        Self::KilocaloriesPerKilogramCelsius,
        Self::JoulesPerMoleKelvin,
        Self::KilojoulesPerKilomoleKelvin,
        Self::BtuPerPoundMoleFahrenheit,
        Self::CaloriesPerMoleKelvin,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::JoulesPerKilogramKelvin => "J/(kg·K)",
            Self::KilojoulesPerKilogramKelvin => "kJ/(kg·K)",
            Self::JoulesPerGramKelvin => "J/(g·K)",
            Self::BtuPerPoundFahrenheit => "BTU/(lb·°F)",
            Self::CaloriesPerGramCelsius => "cal/(g·°C)",
            Self::KilocaloriesPerKilogramCelsius => "kcal/(kg·°C)",
            Self::JoulesPerMoleKelvin => "J/(mol·K)",
            Self::KilojoulesPerKilomoleKelvin => "kJ/(kmol·K)",
            Self::BtuPerPoundMoleFahrenheit => "BTU/(lbmol·°F)",
            Self::CaloriesPerMoleKelvin => "cal/(mol·K)",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::JoulesPerKilogramKelvin => &["j/kg/k", "j/(kg·°c)", "j/kg/c"],
            Self::KilojoulesPerKilogramKelvin => &["kj/kg/k", "kj/(kg·°c)", "kj/kg/c"],
            Self::JoulesPerGramKelvin => &["j/g/k", "j/(g·°c)", "j/g/c"],
            Self::BtuPerPoundFahrenheit => &["btu/lb/f", "btu/(lbm·°f)"],
            Self::CaloriesPerGramCelsius => &["cal/g/c", "cal/(g·k)", "cal/g/k"],
            Self::KilocaloriesPerKilogramCelsius => &["kcal/kg/c", "kcal/(kg·k)", "kcal/kg/k"],
            Self::JoulesPerMoleKelvin => &["j/mol/k", "j/(mol·°c)", "j/mol/c"],
            Self::KilojoulesPerKilomoleKelvin => &["kj/kmol/k", "kj/(kmol·°c)", "kj/kmol/c"],
            Self::BtuPerPoundMoleFahrenheit => &["btu/lbmol/f"],
            Self::CaloriesPerMoleKelvin => &["cal/mol/k", "cal/(mol·°c)", "cal/mol/c"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A non-negative specific heat capacity, stored per kilogram or per mole.
pub struct SpecificHeat {
    basis: AmountBasis,
    value: f64,
}

impl SpecificHeat {
    /// Creates a specific heat capacity from a value and its source unit.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is not finite or is outside the valid range
    /// for `unit`.
    pub fn new(value: f64, unit: SpecificHeatUnit) -> Result<Self, ConversionError> {
        validate_sign(value, Quantity::SpecificHeat, Sign::NonNegative)?;
        let (basis, scale) = unit.basis_and_scale();
        let value = value * scale;
        validate_result(value)?;
        Ok(Self { basis, value })
    }

    /// Returns this value in `unit`. Crossing between mass and molar units
    /// requires `context.molar_mass_g_mol`.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    pub fn in_unit(
        self,
        unit: SpecificHeatUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let (basis, scale) = unit.basis_and_scale();
        let value = rebase(self.value, self.basis, basis, context, SPECIFIC_HEAT_REASON)? / scale;
        validate_result(value)?;
        Ok(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Specific enthalpy (or latent heat) units, on a mass or molar basis.
pub enum SpecificEnthalpyUnit {
    /// Joules per kilogram (`J/kg`).
    JoulesPerKilogram,
    /// Kilojoules per kilogram (`kJ/kg`).
    KilojoulesPerKilogram,
    /// Joules per gram (`J/g`).
    JoulesPerGram,
    /// BTU per pound (`BTU/lb`).
    BtuPerPound,
    /// Calories per gram (`cal/g`).
    CaloriesPerGram,
    /// Kilocalories per kilogram (`kcal/kg`).
    KilocaloriesPerKilogram,
    /// Joules per mole (`J/mol`).
    JoulesPerMole,
    /// Kilojoules per mole (`kJ/mol`).
    KilojoulesPerMole,
    /// Kilojoules per kilomole (`kJ/kmol`).
    KilojoulesPerKilomole,
    /// BTU per pound mole (`BTU/lbmol`).
    BtuPerPoundMole,
    /// Calories per mole (`cal/mol`).
    CaloriesPerMole,
    /// Kilocalories per mole (`kcal/mol`).
    KilocaloriesPerMole,
}

impl SpecificEnthalpyUnit {
    /// Returns the basis and the size of one unit in J/kg or J/mol.
    fn basis_and_scale(self) -> (AmountBasis, f64) {
        use AmountBasis::{Mass, Molar};
        match self {
            Self::JoulesPerKilogram => (Mass, 1.0),
            Self::KilojoulesPerKilogram | Self::JoulesPerGram => (Mass, 1e3),
            Self::BtuPerPound => (Mass, JOULES_PER_BTU / KILOGRAMS_PER_POUND),
            Self::CaloriesPerGram | Self::KilocaloriesPerKilogram => (Mass, KILOCALORIE),
            Self::JoulesPerMole | Self::KilojoulesPerKilomole => (Molar, 1.0),
            Self::KilojoulesPerMole => (Molar, 1e3),
            Self::BtuPerPoundMole => (Molar, JOULES_PER_BTU / MOLES_PER_POUND_MOLE),
            Self::CaloriesPerMole => (Molar, JOULES_PER_CALORIE),
            Self::KilocaloriesPerMole => (Molar, KILOCALORIE),
        }
    }
}

impl UnitSet for SpecificEnthalpyUnit {
    const ALL: &'static [Self] = &[
        Self::JoulesPerKilogram,
        Self::KilojoulesPerKilogram,
        Self::JoulesPerGram,
        Self::BtuPerPound,
        Self::CaloriesPerGram,
        Self::KilocaloriesPerKilogram,
        Self::JoulesPerMole,
        Self::KilojoulesPerMole,
        Self::KilojoulesPerKilomole,
        Self::BtuPerPoundMole,
        Self::CaloriesPerMole,
        Self::KilocaloriesPerMole,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::JoulesPerKilogram => "J/kg",
            Self::KilojoulesPerKilogram => "kJ/kg",
            Self::JoulesPerGram => "J/g",
            Self::BtuPerPound => "BTU/lb",
            Self::CaloriesPerGram => "cal/g",
            Self::KilocaloriesPerKilogram => "kcal/kg",
            Self::JoulesPerMole => "J/mol",
            Self::KilojoulesPerMole => "kJ/mol",
            Self::KilojoulesPerKilomole => "kJ/kmol",
            Self::BtuPerPoundMole => "BTU/lbmol",
            Self::CaloriesPerMole => "cal/mol",
            Self::KilocaloriesPerMole => "kcal/mol",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::BtuPerPound => &["btu/lbm"],
            _ => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A specific enthalpy, stored per kilogram or per mole. It may be negative,
/// since enthalpy is measured from an arbitrary reference state.
pub struct SpecificEnthalpy {
    basis: AmountBasis,
    value: f64,
}

impl SpecificEnthalpy {
    /// Creates a specific enthalpy from a value and its source unit.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::NonFiniteValue`] if `value` is not finite.
    pub fn new(value: f64, unit: SpecificEnthalpyUnit) -> Result<Self, ConversionError> {
        validate_finite(value)?;
        let (basis, scale) = unit.basis_and_scale();
        let value = value * scale;
        validate_result(value)?;
        Ok(Self { basis, value })
    }

    /// Returns this value in `unit`. Crossing between mass and molar units
    /// requires `context.molar_mass_g_mol`.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    pub fn in_unit(
        self,
        unit: SpecificEnthalpyUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let (basis, scale) = unit.basis_and_scale();
        let value = rebase(self.value, self.basis, basis, context, ENTHALPY_REASON)? / scale;
        validate_result(value)?;
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_close, convert_str};

    fn convert(value: f64, from: &str, to: &str) -> Result<f64, ConversionError> {
        convert_str(value, from, to, ConversionContext::default())
    }

    fn water() -> ConversionContext {
        ConversionContext {
            molar_mass_g_mol: Some(18.015),
            ..ConversionContext::default()
        }
    }

    #[test]
    fn converts_heat_transfer_coefficient_conductivity_and_flux() {
        assert_close(
            convert(1.0, "BTU/(h·ft²·°F)", "W/m2K").unwrap(),
            5.678_263_340_863_113,
        );
        assert_close(convert(1.0, "kcal/(h·m²·°C)", "W/m2K").unwrap(), 1.163);
        assert_close(convert(1.0, "cal/(s·cm·°C)", "W/mK").unwrap(), 418.68);
        assert_close(
            convert(1.0, "BTU/hr.ft.F", "W/(m·K)").unwrap(),
            1.730_734_666_295_077,
        );
        assert_close(
            convert(1.0, "BTU/(h·ft²)", "W/m2").unwrap(),
            3.154_590_745_063_442,
        );
        assert_eq!(
            convert(-1.0, "W/m2K", "kW/m2K"),
            Err(ConversionError::NegativeValue(
                Quantity::HeatTransferCoefficient
            ))
        );
    }

    #[test]
    fn specific_heat_uses_degree_size_without_offset() {
        assert_close(convert(1.0, "BTU/(lb·°F)", "kJ/(kg·K)").unwrap(), 4.186_8);
        assert_close(convert(1.0, "cal/(g·°C)", "J/(g·K)").unwrap(), 4.184);
        assert_close(
            convert(1.0, "BTU/(lbmol·°F)", "J/(mol·K)").unwrap(),
            4.186_8,
        );
    }

    #[test]
    fn mass_and_molar_bases_need_molar_mass() {
        assert_eq!(
            convert(4.184, "kJ/(kg·K)", "J/(mol·K)"),
            Err(ConversionError::MissingContext {
                item: ContextItem::MolarMass,
                reason: SPECIFIC_HEAT_REASON,
            })
        );
        let cp = convert_str(4.184, "kJ/kgK", "J/molK", water()).unwrap();
        assert_close(cp, 4.184 * 18.015);
        let latent = convert_str(40.65, "kJ/mol", "kJ/kg", water()).unwrap();
        assert_close(latent, 40.65 * 1_000.0 / 18.015);
    }

    #[test]
    fn converts_enthalpy_including_negative_values() {
        assert_close(convert(1_000.0, "BTU/lb", "kJ/kg").unwrap(), 2_326.0);
        assert_close(convert(-2_326.0, "J/g", "BTU/lb").unwrap(), -1_000.0);
        assert_close(convert(1.0, "kcal/mol", "kJ/mol").unwrap(), 4.184);
    }
}
