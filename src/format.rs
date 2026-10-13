//! Number formatting shared by every front end.
//!
//! Results are shown either with a fixed number of significant figures, or by
//! default with up to [`DEFAULT_SIGNIFICANT_FIGURES`] and trailing zeros
//! removed. Both switch to scientific notation the way C's `%g` does, so a
//! number never shows digits it does not have or hides digits it does.

use std::fmt;

/// The most significant figures [`SignificantFigures`] accepts. An `f64`
/// carries at most 17 meaningful decimal digits.
pub const MAX_SIGNIFICANT_FIGURES: usize = 17;

/// The significant figures shown when none are requested.
pub const DEFAULT_SIGNIFICANT_FIGURES: usize = 10;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
/// A number of significant figures from 1 to [`MAX_SIGNIFICANT_FIGURES`].
///
/// ```
/// use cheme_unit_converter::SignificantFigures;
///
/// assert!(SignificantFigures::new(4).is_some());
/// assert!(SignificantFigures::new(0).is_none());
/// assert!(SignificantFigures::new(18).is_none());
/// ```
pub struct SignificantFigures(usize);

impl SignificantFigures {
    /// Returns `Some` if `digits` is between 1 and [`MAX_SIGNIFICANT_FIGURES`].
    pub fn new(digits: usize) -> Option<Self> {
        (1..=MAX_SIGNIFICANT_FIGURES)
            .contains(&digits)
            .then_some(Self(digits))
    }

    /// Returns the number of significant figures.
    pub fn get(self) -> usize {
        self.0
    }
}

impl fmt::Display for SignificantFigures {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

/// Formats a conversion result for display.
///
/// With `Some(n)`, shows exactly `n` significant figures, keeping trailing
/// zeros because they are significant. With `None`, shows up to
/// [`DEFAULT_SIGNIFICANT_FIGURES`] and removes trailing zeros. Scientific
/// notation is used, as `%g` does, when the decimal exponent is below −4 or
/// at least the number of significant figures. Negative zero prints as zero.
///
/// ```
/// use cheme_unit_converter::{SignificantFigures, format_value};
///
/// assert_eq!(format_value(77.0, None), "77");
/// assert_eq!(format_value(77.0, SignificantFigures::new(4)), "77.00");
/// assert_eq!(format_value(12_345.0, SignificantFigures::new(3)), "1.23e4");
/// assert_eq!(format_value(0.000_012_34, None), "1.234e-5");
/// ```
pub fn format_value(value: f64, significant_figures: Option<SignificantFigures>) -> String {
    match significant_figures {
        Some(digits) => format_significant(value, digits.get()),
        None => trim_insignificant_zeros(&format_significant(value, DEFAULT_SIGNIFICANT_FIGURES)),
    }
}

/// Formats `value` to `digits` significant figures, keeping trailing zeros.
/// `digits` is between 1 and [`MAX_SIGNIFICANT_FIGURES`].
fn format_significant(value: f64, digits: usize) -> String {
    // Adding 0.0 turns −0 into +0, so a zero result never prints as "-0".
    let value = value + 0.0;
    let scientific = format!("{value:.precision$e}", precision = digits - 1);
    let exponent = scientific
        .split_once('e')
        .and_then(|(_, exponent)| exponent.parse::<isize>().ok());
    // Non-finite values have no exponent; they keep Rust's own spelling.
    let (Some(exponent), Ok(digits)) = (exponent, isize::try_from(digits)) else {
        return scientific;
    };
    if (-4..digits).contains(&exponent) {
        let decimals = usize::try_from(digits - 1 - exponent).unwrap_or(0);
        format!("{value:.decimals$}")
    } else {
        scientific
    }
}

/// Removes trailing zeros after the decimal point, in fixed or scientific
/// notation.
fn trim_insignificant_zeros(number: &str) -> String {
    fn trim_fraction(number: &str) -> &str {
        if number.contains('.') {
            number.trim_end_matches('0').trim_end_matches('.')
        } else {
            number
        }
    }
    match number.split_once('e') {
        Some((mantissa, exponent)) => format!("{}e{exponent}", trim_fraction(mantissa)),
        None => trim_fraction(number).to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn exactly(value: f64, digits: usize) -> String {
        format_value(value, SignificantFigures::new(digits))
    }

    #[test]
    fn significant_figures_are_bounded() {
        assert_eq!(SignificantFigures::new(0), None);
        assert_eq!(
            SignificantFigures::new(1).map(SignificantFigures::get),
            Some(1)
        );
        assert_eq!(
            SignificantFigures::new(17).map(SignificantFigures::get),
            Some(17)
        );
        assert_eq!(SignificantFigures::new(18), None);
    }

    #[test]
    fn formats_exact_significant_figures() {
        assert_eq!(exactly(77.0, 3), "77.0");
        assert_eq!(exactly(9.996, 3), "10.0");
        assert_eq!(exactly(12_345.0, 3), "1.23e4");
        assert_eq!(exactly(0.000_123_4, 2), "0.00012");
        assert_eq!(exactly(0.000_012_34, 2), "1.2e-5");
        assert_eq!(exactly(0.0, 3), "0.00");
        assert_eq!(exactly(-0.0, 3), "0.00");
        assert_eq!(exactly(-2.5, 1), "-2");
        assert_eq!(exactly(123.456, 17), "123.45600000000000");
    }

    #[test]
    fn default_format_keeps_ten_significant_figures() {
        assert_eq!(format_value(212.0, None), "212");
        assert_eq!(format_value(0.25, None), "0.25");
        assert_eq!(format_value(1e-12, None), "1e-12");
        assert_eq!(
            format_value(0.000_293_071_070_172_222, None),
            "0.0002930710702"
        );
        assert_eq!(format_value(-0.0, None), "0");
        assert_eq!(format_value(3_600_000.0, None), "3600000");
        assert_eq!(format_value(1e20, None), "1e20");
    }

    #[test]
    fn non_finite_values_do_not_panic() {
        assert_eq!(format_value(f64::INFINITY, None), "inf");
        assert_eq!(format_value(f64::NAN, SignificantFigures::new(3)), "NaN");
    }
}
