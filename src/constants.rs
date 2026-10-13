//! Exact unit definitions and physical constants shared by several quantities.

/// International avoirdupois pound.
pub(crate) const KILOGRAMS_PER_POUND: f64 = 0.453_592_37;
pub(crate) const METRES_PER_FOOT: f64 = 0.304_8;
pub(crate) const METRES_PER_INCH: f64 = 0.025_4;
pub(crate) const CUBIC_METRES_PER_CUBIC_FOOT: f64 = 0.028_316_846_592;
pub(crate) const CUBIC_METRES_PER_US_GALLON: f64 = 0.003_785_411_784;
pub(crate) const GRAMS_PER_KILOGRAM: f64 = 1_000.0;
/// Standard acceleration of gravity, in m/s².
pub(crate) const STANDARD_GRAVITY: f64 = 9.806_65;
/// Molar gas constant, in J/(mol·K).
pub(crate) const MOLAR_GAS_CONSTANT: f64 = 8.314_462_618_153_24;
pub(crate) const SQUARE_METRES_PER_SQUARE_FOOT: f64 = 0.092_903_04;
pub(crate) const MOLES_PER_POUND_MOLE: f64 = KILOGRAMS_PER_POUND * GRAMS_PER_KILOGRAM;
pub(crate) const SECONDS_PER_HOUR: f64 = 3_600.0;
/// Size of one Fahrenheit or Rankine degree, in kelvin.
pub(crate) const KELVIN_PER_FAHRENHEIT_DEGREE: f64 = 5.0 / 9.0;
/// International Table British thermal unit, in joules.
pub(crate) const JOULES_PER_BTU: f64 = 1_055.055_852_62;
/// Thermochemical calorie, in joules.
pub(crate) const JOULES_PER_CALORIE: f64 = 4.184;
/// International Table calorie, in joules.
pub(crate) const JOULES_PER_IT_CALORIE: f64 = 4.186_8;
