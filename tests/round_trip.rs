//! Converts every unit to every other unit of the same quantity and back,
//! checking that the original value is recovered.

use cheme_unit_converter::{
    ConversionContext, ConversionError, Pressure, PressureUnit, Temperature, TemperatureUnit, Unit,
    convert,
};

/// A context that supplies every property, so no conversion is refused for
/// missing context.
fn full_context() -> ConversionContext {
    let none = ConversionContext::new();
    let atmosphere = Pressure::new(101.325, PressureUnit::Kilopascal, &none).unwrap();
    ConversionContext::new()
        .with_density(870.0)
        .with_molar_mass(44.1)
        .with_solvent_molar_mass(18.015)
        .with_reference_density(999.0)
        .with_atmospheric_pressure(atmosphere)
        .with_reference_temperature(Temperature::new(15.0, TemperatureUnit::Celsius).unwrap())
        .with_reference_pressure(atmosphere)
        .with_temperature(Temperature::new(80.0, TemperatureUnit::Celsius).unwrap())
        .with_pressure(Pressure::new(7.0, PressureUnit::Bar, &none).unwrap())
        .with_compressibility_factor(0.97)
}

/// Rounding error from adding and subtracting an atmospheric pressure of
/// about 10⁵ Pa in `f64` (a few units in the last place).
const GAUGE_ROUNDING_PA: f64 = 1e-10;

fn is_gauge(unit: Unit) -> bool {
    matches!(unit, Unit::Pressure(pressure) if pressure.is_gauge())
}

/// The size of one `unit` in pascals; works for gauge and absolute units.
fn pascals_per_unit(unit: Unit, context: ConversionContext) -> f64 {
    let pascal = Unit::Pressure(PressureUnit::Pascal);
    convert(1.0, unit, pascal, context).unwrap() - convert(0.0, unit, pascal, context).unwrap()
}

#[test]
fn every_same_quantity_pair_round_trips() {
    let context = full_context();
    let units = Unit::all();
    let mut checked = 0;
    for &from in &units {
        for &to in units.iter().filter(|to| to.quantity() == from.quantity()) {
            // Values chosen to be valid in every unit, including fractions,
            // gauge pressures, and API gravity.
            for value in [0.0037, 0.42, 0.93] {
                let there = match convert(value, from, to, context) {
                    Ok(there) => there,
                    // A valid value in one unit can be impossible in another
                    // (for example, 0.93 kg/kg exceeds the molality limit for
                    // this context); that is a correct refusal, not a failure.
                    Err(ConversionError::OutOfRange(_)) => continue,
                    Err(error) => panic!("{value} {from} -> {to}: {error}"),
                };
                let back = convert(there, to, from, context)
                    .unwrap_or_else(|error| panic!("{there} {to} -> {from}: {error}"));
                // Gauge conversions add and subtract atmospheric pressure,
                // which loses relative precision on values near zero
                // absolute; allow for that rounding, and nothing more,
                // elsewhere.
                let mut tolerance = 1e-11 * value.abs();
                if is_gauge(from) || is_gauge(to) {
                    tolerance += GAUGE_ROUNDING_PA / pascals_per_unit(from, context);
                }
                assert!(
                    (back - value).abs() <= tolerance,
                    "{value} {from} -> {there} {to} -> {back} {from}"
                );
                checked += 1;
            }
        }
    }
    assert!(checked > 5_000, "only {checked} conversions were checked");
}

#[test]
fn every_unit_converts_to_itself_unchanged() {
    let context = full_context();
    for unit in Unit::all() {
        let result = convert(0.5, unit, unit, context).unwrap();
        assert!((result - 0.5).abs() < 1e-12, "{unit}: {result}");
    }
}
