//! Energy and power.
//!
//! Energy (J, BTU) and power (W, BTU/h) are separate quantities. Power is
//! energy per unit time, so relating them needs a time interval and is a
//! calculation, not a unit conversion.
//!
//! Plain `cal` and `kcal` are thermochemical calories (4.184 J), the usual
//! chemistry convention. `cal_IT` and `kcal_IT` are International Table
//! calories (4.1868 J), found in steam tables and older engineering data.
//! `kcal/h` is a heat-duty unit from engineering practice, so it uses the
//! International Table kilocalorie: 1 kcal/h = 1.163 W.
//! BTU is the International Table BTU (1055.05585262 J).

use crate::constants::{
    JOULES_PER_BTU, JOULES_PER_CALORIE, JOULES_PER_IT_CALORIE, KILOGRAMS_PER_POUND,
    METRES_PER_FOOT, SECONDS_PER_HOUR, STANDARD_GRAVITY,
};
use crate::error::{Quantity, Sign};
use crate::linear::{LinearUnit, linear_quantity};
use crate::unit::UnitSet;

/// Work done by one pound-force acting through one foot, in joules.
const JOULES_PER_FOOT_POUND_FORCE: f64 = METRES_PER_FOOT * KILOGRAMS_PER_POUND * STANDARD_GRAVITY;
/// Mechanical horsepower: 550 ft·lbf/s.
const WATTS_PER_HORSEPOWER: f64 = 550.0 * JOULES_PER_FOOT_POUND_FORCE;
/// Metric horsepower (PS): 75 kgf·m/s.
const WATTS_PER_METRIC_HORSEPOWER: f64 = 75.0 * STANDARD_GRAVITY;
/// Ton of refrigeration: 12,000 BTU/h.
const WATTS_PER_TON_OF_REFRIGERATION: f64 = 12_000.0 * JOULES_PER_BTU / SECONDS_PER_HOUR;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Energy units.
pub enum EnergyUnit {
    /// Joule (`J`).
    Joule,
    /// Kilojoule (`kJ`).
    Kilojoule,
    /// Megajoule (`MJ`).
    Megajoule,
    /// Gigajoule (`GJ`).
    Gigajoule,
    /// Kilowatt hour (`kWh`).
    KilowattHour,
    /// Thermochemical calorie, 4.184 J (`cal`).
    Calorie,
    /// Thermochemical kilocalorie, 4184 J (`kcal`).
    Kilocalorie,
    /// International Table calorie, 4.1868 J (`cal_IT`).
    InternationalTableCalorie,
    /// International Table kilocalorie, 4186.8 J (`kcal_IT`).
    InternationalTableKilocalorie,
    /// International Table British thermal unit (`BTU`).
    Btu,
    /// Million BTU (`MMBTU`).
    MillionBtu,
    /// Foot-pound force (`ft·lbf`).
    FootPoundForce,
}

impl LinearUnit for EnergyUnit {
    fn si_per_unit(self) -> f64 {
        match self {
            Self::Joule => 1.0,
            Self::Kilojoule => 1e3,
            Self::Megajoule => 1e6,
            Self::Gigajoule => 1e9,
            Self::KilowattHour => 1e3 * SECONDS_PER_HOUR,
            Self::Calorie => JOULES_PER_CALORIE,
            Self::Kilocalorie => 1e3 * JOULES_PER_CALORIE,
            Self::InternationalTableCalorie => JOULES_PER_IT_CALORIE,
            Self::InternationalTableKilocalorie => 1e3 * JOULES_PER_IT_CALORIE,
            Self::Btu => JOULES_PER_BTU,
            Self::MillionBtu => 1e6 * JOULES_PER_BTU,
            Self::FootPoundForce => JOULES_PER_FOOT_POUND_FORCE,
        }
    }
}

impl UnitSet for EnergyUnit {
    const ALL: &'static [Self] = &[
        Self::Joule,
        Self::Kilojoule,
        Self::Megajoule,
        Self::Gigajoule,
        Self::KilowattHour,
        Self::Calorie,
        Self::Kilocalorie,
        Self::InternationalTableCalorie,
        Self::InternationalTableKilocalorie,
        Self::Btu,
        Self::MillionBtu,
        Self::FootPoundForce,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Joule => "J",
            Self::Kilojoule => "kJ",
            Self::Megajoule => "MJ",
            Self::Gigajoule => "GJ",
            Self::KilowattHour => "kWh",
            Self::Calorie => "cal",
            Self::Kilocalorie => "kcal",
            Self::InternationalTableCalorie => "cal_IT",
            Self::InternationalTableKilocalorie => "kcal_IT",
            Self::Btu => "BTU",
            Self::MillionBtu => "MMBTU",
            Self::FootPoundForce => "ft·lbf",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Joule => &["joule"],
            Self::Kilojoule => &[],
            Self::Megajoule => &[],
            Self::Gigajoule => &[],
            Self::KilowattHour => &[],
            Self::Calorie => &["cal_th", "calorie"],
            Self::Kilocalorie => &["kcal_th"],
            Self::InternationalTableCalorie => &[],
            Self::InternationalTableKilocalorie => &[],
            Self::Btu => &["btu_it"],
            Self::MillionBtu => &[],
            Self::FootPoundForce => &[],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Power and heat-duty units.
pub enum PowerUnit {
    /// Watt (`W`).
    Watt,
    /// Kilowatt (`kW`).
    Kilowatt,
    /// Megawatt (`MW`).
    Megawatt,
    /// Kilojoule per hour (`kJ/h`).
    KilojoulePerHour,
    /// Megajoule per hour (`MJ/h`).
    MegajoulePerHour,
    /// Mechanical horsepower, 550 ft·lbf/s (`hp`).
    Horsepower,
    /// Metric horsepower, 75 kgf·m/s (`hp(metric)`).
    MetricHorsepower,
    /// BTU per hour (`BTU/h`).
    BtuPerHour,
    /// Million BTU per hour (`MMBTU/h`).
    MillionBtuPerHour,
    /// International Table kilocalorie per hour (`kcal/h`).
    KilocaloriePerHour,
    /// Ton of refrigeration (`TR`).
    TonOfRefrigeration,
}

impl LinearUnit for PowerUnit {
    fn si_per_unit(self) -> f64 {
        match self {
            Self::Watt => 1.0,
            Self::Kilowatt => 1e3,
            Self::Megawatt => 1e6,
            Self::KilojoulePerHour => 1e3 / SECONDS_PER_HOUR,
            Self::MegajoulePerHour => 1e6 / SECONDS_PER_HOUR,
            Self::Horsepower => WATTS_PER_HORSEPOWER,
            Self::MetricHorsepower => WATTS_PER_METRIC_HORSEPOWER,
            Self::BtuPerHour => JOULES_PER_BTU / SECONDS_PER_HOUR,
            Self::MillionBtuPerHour => 1e6 * JOULES_PER_BTU / SECONDS_PER_HOUR,
            Self::KilocaloriePerHour => 1e3 * JOULES_PER_IT_CALORIE / SECONDS_PER_HOUR,
            Self::TonOfRefrigeration => WATTS_PER_TON_OF_REFRIGERATION,
        }
    }
}

impl UnitSet for PowerUnit {
    const ALL: &'static [Self] = &[
        Self::Watt,
        Self::Kilowatt,
        Self::Megawatt,
        Self::KilojoulePerHour,
        Self::MegajoulePerHour,
        Self::Horsepower,
        Self::MetricHorsepower,
        Self::BtuPerHour,
        Self::MillionBtuPerHour,
        Self::KilocaloriePerHour,
        Self::TonOfRefrigeration,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::Watt => "W",
            Self::Kilowatt => "kW",
            Self::Megawatt => "MW",
            Self::KilojoulePerHour => "kJ/h",
            Self::MegajoulePerHour => "MJ/h",
            Self::Horsepower => "hp",
            Self::MetricHorsepower => "hp(metric)",
            Self::BtuPerHour => "BTU/h",
            Self::MillionBtuPerHour => "MMBTU/h",
            Self::KilocaloriePerHour => "kcal/h",
            Self::TonOfRefrigeration => "TR",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::Watt => &["watt", "j/s"],
            Self::Kilowatt => &["kj/s"],
            Self::Megawatt => &["mj/s"],
            Self::KilojoulePerHour => &[],
            Self::MegajoulePerHour => &[],
            Self::Horsepower => &[],
            Self::MetricHorsepower => &["ps"],
            Self::BtuPerHour => &["btuh"],
            Self::MillionBtuPerHour => &[],
            Self::KilocaloriePerHour => &[],
            Self::TonOfRefrigeration => &["rt", "ton-refrigeration", "tons-refrigeration"],
        }
    }
}

linear_quantity! {
    /// An amount of energy (or heat) stored in joules. It may be negative.
    Energy(EnergyUnit, Quantity::Energy, Sign::Any)
}

linear_quantity! {
    /// A power or heat duty stored in watts. It may be negative (heat removed).
    Power(PowerUnit, Quantity::Power, Sign::Any)
}

#[cfg(test)]
mod tests {
    use crate::test_support::{assert_close, convert_str};
    use crate::{ConversionContext, ConversionError, Quantity};

    fn convert(value: f64, from: &str, to: &str) -> Result<f64, ConversionError> {
        convert_str(value, from, to, ConversionContext::default())
    }

    #[test]
    fn converts_energy() {
        assert_close(convert(1.0, "kWh", "MJ").unwrap(), 3.6);
        assert_close(convert(1.0, "BTU", "kJ").unwrap(), 1.055_055_852_62);
        assert_close(convert(1.0, "kcal", "kJ").unwrap(), 4.184);
        assert_close(convert(1.0, "kcal_IT", "kJ").unwrap(), 4.186_8);
        assert_close(
            convert(1.0, "ft.lbf", "J").unwrap(),
            1.355_817_948_331_400_4,
        );
    }

    #[test]
    fn converts_power_and_heat_duty() {
        assert_close(convert(1.0, "hp", "kW").unwrap(), 0.745_699_871_582_270_2);
        assert_close(convert(1.0, "TR", "kW").unwrap(), 3.516_852_842_066_667);
        assert_close(
            convert(1.0, "MMBTU/h", "kW").unwrap(),
            293.071_070_172_222_2,
        );
        assert_close(convert(-3_600.0, "kJ/h", "W").unwrap(), -1_000.0);
        assert_close(convert(1.0, "kcal/h", "W").unwrap(), 1.163);
    }

    #[test]
    fn refuses_to_mix_energy_and_power() {
        let error = convert(1.0, "kWh", "kW").unwrap_err();
        assert_eq!(
            error,
            ConversionError::IncompatibleUnits {
                from: Quantity::Energy,
                to: Quantity::Power,
            }
        );
        assert!(error.to_string().contains("energy per unit time"));
    }
}
