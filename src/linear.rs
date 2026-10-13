//! Shared implementation for quantities whose units differ only by a factor.

use crate::unit::UnitSet;

/// Implemented by unit enums whose units differ only by a constant factor.
pub(crate) trait LinearUnit: UnitSet {
    /// The size of one unit in the quantity's SI unit.
    fn si_per_unit(self) -> f64;
}

/// Defines a newtype that stores a quantity in its SI unit and converts it
/// with scale factors alone.
macro_rules! linear_quantity {
    (
        $(#[$meta:meta])*
        $name:ident($unit:ty, $quantity:expr, $sign:expr)
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq)]
        pub struct $name {
            si: f64,
        }

        impl $name {
            /// Creates a value from a number and its unit.
            ///
            /// # Errors
            ///
            /// Returns an error if `value` is not finite or is outside the valid range
            /// for `unit`.
            pub fn new(value: f64, unit: $unit) -> Result<Self, crate::ConversionError> {
                crate::error::validate_sign(value, $quantity, $sign)?;
                let si = value * crate::linear::LinearUnit::si_per_unit(unit);
                crate::error::validate_result(si)?;
                Ok(Self { si })
            }

            /// Returns this value expressed in `unit`.
            ///
            /// # Errors
            ///
            /// Returns [`crate::ConversionError::ResultOverflow`] if the result overflows.
            pub fn in_unit(self, unit: $unit) -> Result<f64, crate::ConversionError> {
                let value = self.si / crate::linear::LinearUnit::si_per_unit(unit);
                crate::error::validate_result(value)?;
                Ok(value)
            }
        }
    };
}

pub(crate) use linear_quantity;
