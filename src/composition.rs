//! Solution and mixture composition.
//!
//! Composition units fall into five bases that are not interchangeable
//! without mixture properties:
//!
//! - mass fraction (kg/kg, wt%, ppm, ppb): no context between these
//! - mass per volume (mg/L, g/L): density relates it to mass fraction
//! - molar concentration (mol/L, mmol/L): density and solute molar mass
//! - mole fraction (mol/mol, mol%, ppm(mol)): solute and solvent molar masses
//! - molality (mol/kg solvent): solute molar mass
//!
//! Everything that is not the solute counts as the solvent. For a
//! multicomponent remainder, supply its average molar mass as the solvent
//! molar mass. ppm and ppb are mass-based; use ppm(mol) for mole-based ppm.

use crate::constants::GRAMS_PER_KILOGRAM;
use crate::context::ConversionContext;
use crate::error::{
    ContextItem, ConversionError, Quantity, validate_non_negative, validate_result,
};
use crate::unit::UnitSet;

const MASS_PER_VOLUME: &str = "converting mass-per-volume concentration (mg/L, g/L)";
const MOLAR_CONCENTRATION: &str = "converting molar concentration (mol/L, mmol/L)";
const MOLE_FRACTION: &str = "converting mole-based composition (mol/mol, mol%, ppm(mol))";
const MOLALITY: &str = "converting molality (mol/kg solvent)";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CompositionBasis {
    /// kg solute / kg mixture.
    MassFraction,
    /// kg solute / m³ mixture.
    MassConcentration,
    /// mol solute / m³ mixture.
    MolarConcentration,
    /// mol solute / mol mixture.
    MoleFraction,
    /// mol solute / kg solvent.
    Molality,
}

impl CompositionBasis {
    fn is_fraction(self) -> bool {
        matches!(self, Self::MassFraction | Self::MoleFraction)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
/// Composition and concentration units supported by the converter.
pub enum ConcentrationUnit {
    /// Mass fraction (`kg/kg`).
    MassFraction,
    /// Weight percent (`wt%`).
    WeightPercent,
    /// Parts per million (`ppm`).
    PartsPerMillion,
    /// Parts per billion (`ppb`).
    PartsPerBillion,
    /// Milligrams per litre (`mg/L`).
    MilligramsPerLitre,
    /// Grams per litre (`g/L`).
    GramsPerLitre,
    /// Moles per litre (`mol/L`).
    MolesPerLitre,
    /// Millimoles per litre (`mmol/L`).
    MillimolesPerLitre,
    /// Mole fraction (`mol/mol`).
    MoleFraction,
    /// Mole percent (`mol%`).
    MolePercent,
    /// Molar parts per million (`ppm(mol)`).
    MolarPartsPerMillion,
    /// Molality (`mol/kg`).
    Molality,
}

impl ConcentrationUnit {
    fn basis_and_scale(self) -> (CompositionBasis, f64) {
        use CompositionBasis::{
            MassConcentration, MassFraction, Molality, MolarConcentration, MoleFraction,
        };
        match self {
            Self::MassFraction => (MassFraction, 1.0),
            Self::WeightPercent => (MassFraction, 1e-2),
            Self::PartsPerMillion => (MassFraction, 1e-6),
            Self::PartsPerBillion => (MassFraction, 1e-9),
            Self::MilligramsPerLitre => (MassConcentration, 1e-3),
            Self::GramsPerLitre => (MassConcentration, 1.0),
            Self::MolesPerLitre => (MolarConcentration, 1_000.0),
            Self::MillimolesPerLitre => (MolarConcentration, 1.0),
            Self::MoleFraction => (MoleFraction, 1.0),
            Self::MolePercent => (MoleFraction, 1e-2),
            Self::MolarPartsPerMillion => (MoleFraction, 1e-6),
            Self::Molality => (Molality, 1.0),
        }
    }
}

impl UnitSet for ConcentrationUnit {
    const ALL: &'static [Self] = &[
        Self::MassFraction,
        Self::WeightPercent,
        Self::PartsPerMillion,
        Self::PartsPerBillion,
        Self::MilligramsPerLitre,
        Self::GramsPerLitre,
        Self::MolesPerLitre,
        Self::MillimolesPerLitre,
        Self::MoleFraction,
        Self::MolePercent,
        Self::MolarPartsPerMillion,
        Self::Molality,
    ];

    fn label(self) -> &'static str {
        match self {
            Self::MassFraction => "kg/kg",
            Self::WeightPercent => "wt%",
            Self::PartsPerMillion => "ppm",
            Self::PartsPerBillion => "ppb",
            Self::MilligramsPerLitre => "mg/L",
            Self::GramsPerLitre => "g/L",
            Self::MolesPerLitre => "mol/L",
            Self::MillimolesPerLitre => "mmol/L",
            Self::MoleFraction => "mol/mol",
            Self::MolePercent => "mol%",
            Self::MolarPartsPerMillion => "ppm(mol)",
            Self::Molality => "mol/kg",
        }
    }

    fn aliases(self) -> &'static [&'static str] {
        match self {
            Self::MassFraction => &["mass-fraction", "massfrac"],
            Self::WeightPercent => &["w%", "mass%", "wtpercent"],
            Self::PartsPerMillion => &["ppmw", "ppm(mass)", "mg/kg"],
            Self::PartsPerBillion => &["ppbw", "µg/kg", "ug/kg"],
            Self::MilligramsPerLitre => &["mg/litre", "mg/liter"],
            Self::GramsPerLitre => &["g/litre", "g/liter"],
            Self::MolesPerLitre => &["mol/litre", "mol/liter", "mol/dm3", "molar"],
            Self::MillimolesPerLitre => &["mmol/litre", "mmol/liter"],
            Self::MoleFraction => &["x", "mole-fraction", "molfrac"],
            Self::MolePercent => &["mole%", "molpercent"],
            Self::MolarPartsPerMillion => &[],
            Self::Molality => &["molal", "molality"],
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A non-negative composition stored in the SI unit of its basis.
pub struct Concentration {
    basis: CompositionBasis,
    value: f64,
}

impl Concentration {
    /// Creates a composition from a value and its source unit.
    ///
    /// Fractions above 1 (100%, 10⁶ ppm, ...) are rejected.
    ///
    /// # Errors
    ///
    /// Returns an error if `value` is not finite or is outside the valid range
    /// for `unit`.
    pub fn new(value: f64, unit: ConcentrationUnit) -> Result<Self, ConversionError> {
        validate_non_negative(value, Quantity::Composition)?;
        let (basis, scale) = unit.basis_and_scale();
        let value = value * scale;
        validate_result(value)?;
        check_fraction(value, basis, unit.label())?;
        Ok(Self { basis, value })
    }

    /// Returns this composition in `unit`.
    ///
    /// Crossing bases needs whichever of density, molar mass, and solvent
    /// molar mass relates them; see the module documentation.
    ///
    /// # Errors
    ///
    /// Returns [`ConversionError::MissingContext`] or
    /// [`ConversionError::InvalidContext`] when reaching `unit` needs a property
    /// that `context` lacks or holds as a non-positive value, and
    /// [`ConversionError::ResultOverflow`] if the result overflows.
    /// Also returns [`ConversionError::OutOfRange`] when the result is not a
    /// physically possible composition.
    pub fn in_unit(
        self,
        unit: ConcentrationUnit,
        context: &ConversionContext,
    ) -> Result<f64, ConversionError> {
        let (basis, scale) = unit.basis_and_scale();
        let value = convert_basis(self.value, self.basis, basis, context)?;
        check_fraction(value, basis, unit.label())?;
        let value = value / scale;
        validate_result(value)?;
        Ok(value)
    }
}

fn check_fraction(
    value: f64,
    basis: CompositionBasis,
    unit: &'static str,
) -> Result<(), ConversionError> {
    if basis.is_fraction() && value > 1.0 {
        Err(ConversionError::OutOfRange(unit))
    } else {
        Ok(())
    }
}

fn convert_basis(
    value: f64,
    from: CompositionBasis,
    to: CompositionBasis,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    use CompositionBasis::{
        MassConcentration, MassFraction, Molality, MolarConcentration, MoleFraction,
    };
    // Direct routes below need fewer properties than going via mass fraction.
    match (from, to) {
        _ if from == to => Ok(value),
        (MassConcentration, MolarConcentration) => {
            Ok(value * GRAMS_PER_KILOGRAM / solute_molar_mass(context, MOLAR_CONCENTRATION)?)
        }
        (MolarConcentration, MassConcentration) => {
            Ok(value * solute_molar_mass(context, MOLAR_CONCENTRATION)? / GRAMS_PER_KILOGRAM)
        }
        (MoleFraction, Molality) => {
            if value >= 1.0 {
                return Err(ConversionError::OutOfRange("mol/kg"));
            }
            Ok(value * GRAMS_PER_KILOGRAM
                / ((1.0 - value) * solvent_molar_mass(context, MOLALITY)?))
        }
        (Molality, MoleFraction) => {
            let solute_per_solvent_mole =
                value * solvent_molar_mass(context, MOLALITY)? / GRAMS_PER_KILOGRAM;
            Ok(solute_per_solvent_mole / (1.0 + solute_per_solvent_mole))
        }
        _ => {
            let mass_fraction = to_mass_fraction(value, from, context)?;
            check_fraction(mass_fraction, MassFraction, "mass fraction")?;
            from_mass_fraction(mass_fraction, to, context)
        }
    }
}

fn to_mass_fraction(
    value: f64,
    basis: CompositionBasis,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    Ok(match basis {
        CompositionBasis::MassFraction => value,
        CompositionBasis::MassConcentration => value / density(context, MASS_PER_VOLUME)?,
        CompositionBasis::MolarConcentration => {
            value * solute_molar_mass(context, MOLAR_CONCENTRATION)?
                / GRAMS_PER_KILOGRAM
                / density(context, MOLAR_CONCENTRATION)?
        }
        CompositionBasis::MoleFraction => {
            let solute = value * solute_molar_mass(context, MOLE_FRACTION)?;
            let solvent = (1.0 - value) * solvent_molar_mass(context, MOLE_FRACTION)?;
            solute / (solute + solvent)
        }
        CompositionBasis::Molality => {
            let solute_per_solvent =
                value * solute_molar_mass(context, MOLALITY)? / GRAMS_PER_KILOGRAM;
            solute_per_solvent / (1.0 + solute_per_solvent)
        }
    })
}

fn from_mass_fraction(
    mass_fraction: f64,
    basis: CompositionBasis,
    context: &ConversionContext,
) -> Result<f64, ConversionError> {
    Ok(match basis {
        CompositionBasis::MassFraction => mass_fraction,
        CompositionBasis::MassConcentration => mass_fraction * density(context, MASS_PER_VOLUME)?,
        CompositionBasis::MolarConcentration => {
            mass_fraction * density(context, MOLAR_CONCENTRATION)? * GRAMS_PER_KILOGRAM
                / solute_molar_mass(context, MOLAR_CONCENTRATION)?
        }
        CompositionBasis::MoleFraction => {
            let solute = mass_fraction / solute_molar_mass(context, MOLE_FRACTION)?;
            let solvent = (1.0 - mass_fraction) / solvent_molar_mass(context, MOLE_FRACTION)?;
            solute / (solute + solvent)
        }
        CompositionBasis::Molality => {
            if mass_fraction >= 1.0 {
                return Err(ConversionError::OutOfRange("mol/kg"));
            }
            mass_fraction * GRAMS_PER_KILOGRAM
                / ((1.0 - mass_fraction) * solute_molar_mass(context, MOLALITY)?)
        }
    })
}

fn density(context: &ConversionContext, reason: &'static str) -> Result<f64, ConversionError> {
    context.require(ContextItem::Density, reason)
}

fn solute_molar_mass(
    context: &ConversionContext,
    reason: &'static str,
) -> Result<f64, ConversionError> {
    context.require(ContextItem::MolarMass, reason)
}

fn solvent_molar_mass(
    context: &ConversionContext,
    reason: &'static str,
) -> Result<f64, ConversionError> {
    context.require(ContextItem::SolventMolarMass, reason)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support::{assert_close, convert_str};

    fn context(
        density: Option<f64>,
        solute: Option<f64>,
        solvent: Option<f64>,
    ) -> ConversionContext {
        ConversionContext {
            density_kg_m3: density,
            molar_mass_g_mol: solute,
            solvent_molar_mass_g_mol: solvent,
            ..ConversionContext::default()
        }
    }

    #[test]
    fn rescales_within_a_basis_without_context() {
        let none = ConversionContext::default();
        assert_close(convert_str(0.25, "wt%", "ppm", none).unwrap(), 2_500.0);
        assert_close(convert_str(1.0, "mol/L", "mmol/L", none).unwrap(), 1_000.0);
        assert_close(convert_str(1.0, "g/L", "mg/L", none).unwrap(), 1_000.0);
        assert_close(convert_str(0.5, "x", "mol%", none).unwrap(), 50.0);
    }

    #[test]
    fn per_volume_conversions_use_density_and_molar_mass() {
        let brine = context(Some(1_000.0), Some(58.44), None);
        assert_close(convert_str(1.0, "mol/L", "mg/L", brine).unwrap(), 58_440.0);
        assert_close(convert_str(1.0, "mg/L", "ppm", brine).unwrap(), 1.0);
        // Mass and molar concentration share a volume basis, so density is not needed.
        let solute_only = context(None, Some(58.44), None);
        assert_close(
            convert_str(58.44, "g/L", "mol/L", solute_only).unwrap(),
            1.0,
        );
        assert_eq!(
            convert_str(1.0, "mol/L", "wt%", context(Some(0.0), Some(58.44), None)),
            Err(ConversionError::InvalidContext(ContextItem::Density))
        );
    }

    #[test]
    fn mole_fraction_needs_both_molar_masses() {
        let ethanol_water = context(None, Some(46.07), Some(18.015));
        let mass_fraction = convert_str(0.5, "x", "kg/kg", ethanol_water).unwrap();
        assert_close(mass_fraction, 46.07 / (46.07 + 18.015));
        assert_close(
            convert_str(mass_fraction, "kg/kg", "mol%", ethanol_water).unwrap(),
            50.0,
        );
        assert!(matches!(
            convert_str(0.5, "x", "wt%", context(None, Some(46.07), None)),
            Err(ConversionError::MissingContext {
                item: ContextItem::SolventMolarMass,
                ..
            })
        ));
    }

    #[test]
    fn converts_molality() {
        let solute = context(None, Some(100.0), None);
        assert_close(convert_str(50.0, "wt%", "mol/kg", solute).unwrap(), 10.0);
        assert_close(convert_str(10.0, "mol/kg", "wt%", solute).unwrap(), 50.0);
        // Mole fraction and molality are related through the solvent alone.
        let water = context(None, None, Some(18.0));
        assert_close(
            convert_str(0.5, "x", "mol/kg", water).unwrap(),
            1_000.0 / 18.0,
        );
        assert_eq!(
            convert_str(100.0, "wt%", "mol/kg", solute),
            Err(ConversionError::OutOfRange("mol/kg"))
        );
    }

    #[test]
    fn rejects_fractions_above_one_and_impossible_mixtures() {
        let none = ConversionContext::default();
        assert_eq!(
            convert_str(101.0, "wt%", "ppm", none),
            Err(ConversionError::OutOfRange("wt%"))
        );
        let water = context(Some(1_000.0), None, None);
        assert_eq!(
            convert_str(2_000.0, "g/L", "wt%", water),
            Err(ConversionError::OutOfRange("mass fraction"))
        );
    }
}
