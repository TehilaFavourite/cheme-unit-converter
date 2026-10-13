//! Physical properties and reference conditions supplied by the caller.

use crate::error::{ContextItem, ConversionError};
use crate::pressure::Pressure;
use crate::temperature::Temperature;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[non_exhaustive]
/// Physical properties and reference conditions for conversions whose bases
/// differ.
///
/// Every field is optional; supply only what a conversion needs. Nothing is
/// defaulted, so a conversion that needs a missing value fails with
/// [`ConversionError::MissingContext`] rather than assuming one.
///
/// Build a context with the `with_*` methods:
///
/// ```
/// use cheme_unit_converter::ConversionContext;
///
/// let context = ConversionContext::new()
///     .with_density(998.2)
///     .with_molar_mass(18.015);
/// assert_eq!(context.density_kg_m3, Some(998.2));
/// ```
pub struct ConversionContext {
    /// Fluid, solution, or mixture density, in kg/m³.
    pub density_kg_m3: Option<f64>,
    /// Molar mass of the flowing substance or of the solute, in g/mol.
    pub molar_mass_g_mol: Option<f64>,
    /// Molar mass of the solvent (or average of the rest of the mixture), in g/mol.
    pub solvent_molar_mass_g_mol: Option<f64>,
    /// Density of the reference substance for specific gravity and °API, in kg/m³.
    pub reference_density_kg_m3: Option<f64>,
    /// Local atmospheric pressure, used to convert gauge pressure.
    pub atmospheric_pressure: Option<Pressure>,
    /// Temperature that defines standard (or normal) gas volume.
    pub reference_temperature: Option<Temperature>,
    /// Pressure that defines standard (or normal) gas volume.
    pub reference_pressure: Option<Pressure>,
    /// Actual temperature of a gas stream.
    pub temperature: Option<Temperature>,
    /// Actual absolute pressure of a gas stream.
    pub pressure: Option<Pressure>,
    /// Compressibility factor Z of a gas stream at its actual conditions
    /// (1 for an ideal gas).
    pub compressibility_factor: Option<f64>,
}

impl ConversionContext {
    /// Returns an empty context, the same as [`ConversionContext::default`].
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the fluid, solution, or mixture density, in kg/m³.
    #[must_use]
    pub fn with_density(mut self, kg_m3: f64) -> Self {
        self.density_kg_m3 = Some(kg_m3);
        self
    }

    /// Sets the molar mass of the flowing substance or solute, in g/mol.
    #[must_use]
    pub fn with_molar_mass(mut self, g_mol: f64) -> Self {
        self.molar_mass_g_mol = Some(g_mol);
        self
    }

    /// Sets the molar mass of the solvent or rest of the mixture, in g/mol.
    #[must_use]
    pub fn with_solvent_molar_mass(mut self, g_mol: f64) -> Self {
        self.solvent_molar_mass_g_mol = Some(g_mol);
        self
    }

    /// Sets the reference density for specific gravity and °API, in kg/m³.
    #[must_use]
    pub fn with_reference_density(mut self, kg_m3: f64) -> Self {
        self.reference_density_kg_m3 = Some(kg_m3);
        self
    }

    /// Sets the local atmospheric pressure used for gauge units.
    #[must_use]
    pub fn with_atmospheric_pressure(mut self, pressure: Pressure) -> Self {
        self.atmospheric_pressure = Some(pressure);
        self
    }

    /// Sets the temperature that defines standard gas volume.
    #[must_use]
    pub fn with_reference_temperature(mut self, temperature: Temperature) -> Self {
        self.reference_temperature = Some(temperature);
        self
    }

    /// Sets the pressure that defines standard gas volume.
    #[must_use]
    pub fn with_reference_pressure(mut self, pressure: Pressure) -> Self {
        self.reference_pressure = Some(pressure);
        self
    }

    /// Sets the actual temperature of a gas stream.
    #[must_use]
    pub fn with_temperature(mut self, temperature: Temperature) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Sets the actual absolute pressure of a gas stream.
    #[must_use]
    pub fn with_pressure(mut self, pressure: Pressure) -> Self {
        self.pressure = Some(pressure);
        self
    }

    /// Sets the compressibility factor Z at the actual gas conditions.
    #[must_use]
    pub fn with_compressibility_factor(mut self, z: f64) -> Self {
        self.compressibility_factor = Some(z);
        self
    }

    /// Returns `item` as a positive number in kg/m³, g/mol, Pa, K, or (for Z)
    /// a plain ratio.
    ///
    /// `reason` describes the conversion step and is reported if the item is
    /// missing.
    pub(crate) fn require(
        &self,
        item: ContextItem,
        reason: &'static str,
    ) -> Result<f64, ConversionError> {
        let value = match item {
            ContextItem::Density => self.density_kg_m3,
            ContextItem::MolarMass => self.molar_mass_g_mol,
            ContextItem::SolventMolarMass => self.solvent_molar_mass_g_mol,
            ContextItem::ReferenceDensity => self.reference_density_kg_m3,
            ContextItem::AtmosphericPressure => self.atmospheric_pressure.map(Pressure::pascals),
            ContextItem::ReferenceTemperature => {
                self.reference_temperature.map(Temperature::kelvin)
            }
            ContextItem::ReferencePressure => self.reference_pressure.map(Pressure::pascals),
            ContextItem::OperatingTemperature => self.temperature.map(Temperature::kelvin),
            ContextItem::OperatingPressure => self.pressure.map(Pressure::pascals),
            ContextItem::CompressibilityFactor => self.compressibility_factor,
        };
        let value = value.ok_or(ConversionError::MissingContext { item, reason })?;
        if value.is_finite() && value > 0.0 {
            Ok(value)
        } else {
            Err(ConversionError::InvalidContext(item))
        }
    }
}
