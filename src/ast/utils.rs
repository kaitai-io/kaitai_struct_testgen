use std::cmp::Ordering;
use std::hash::{Hash, Hasher};

#[derive(Copy, Clone, Debug, PartialEq, PartialOrd)]
pub struct PositiveFiniteF64 {
    value: f64,
}

// PositiveFiniteF64 doesn't permit NaN values, so equality comparison is an equivalence relation
impl Eq for PositiveFiniteF64 {}

impl Hash for PositiveFiniteF64 {
    // inspired by https://github.com/reem/rust-ordered-float/blob/v3.7.0/src/lib.rs#L159-L169
    fn hash<H: Hasher>(&self, state: &mut H) {
        // we only have one zero (with the positive sign) and don't have NaNs, so it should be fine
        // to just hash the raw bits
        self.value.to_bits().hash(state)
    }
}

// Unfortunately, Clippy (as of version rust-1.70.0) doesn't recognize this trivial implementation
// as correct, even though it has been suggested in
// https://github.com/rust-lang/rust-clippy/issues/1621#issuecomment-450339758
#[allow(clippy::derive_ord_xor_partial_ord)]
impl Ord for PositiveFiniteF64 {
    fn cmp(&self, other: &Self) -> Ordering {
        // PositiveFiniteF64 doesn't permit NaN values, so partial_cmp will always give an ordering
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum InvalidFloatError {
    Negative,
    NonFinite,
}

impl TryFrom<f64> for PositiveFiniteF64 {
    type Error = InvalidFloatError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            if value.is_sign_positive() {
                Ok(Self { value })
            } else {
                Err(InvalidFloatError::Negative)
            }
        } else {
            Err(InvalidFloatError::NonFinite)
        }
    }
}

impl PositiveFiniteF64 {
    pub fn value(&self) -> f64 {
        self.value
    }
}

#[cfg(test)]
mod tests {
    use super::{InvalidFloatError, PositiveFiniteF64};

    #[test]
    fn float_pos_nan() {
        let value = f64::NAN;
        assert!(value.is_nan());
        assert!(value.is_sign_positive());
        err_non_finite(value);
    }

    #[test]
    fn float_neg_nan() {
        let value = -f64::NAN;
        assert!(value.is_nan());
        assert!(value.is_sign_negative());
        err_non_finite(value);
    }

    #[test]
    fn float_pos_infinity() {
        err_non_finite(f64::INFINITY);
    }

    #[test]
    fn float_neg_infinity() {
        err_non_finite(f64::NEG_INFINITY);
    }

    #[test]
    fn float_pos_zero() {
        ok(0.0);
    }

    #[test]
    fn float_neg_zero() {
        err_negative(-0.0);
    }

    /// https://en.wikipedia.org/wiki/Double-precision_floating-point_format#Double-precision_examples
    /// (Max. subnormal double)
    fn pos_max_subnormal() -> f64 {
        ((f64::MIN_EXP - 1) as f64).exp2()
            * (1.0 - (-((f64::MANTISSA_DIGITS - 1) as i32) as f64).exp2())
    }

    /// https://en.wikipedia.org/wiki/Double-precision_floating-point_format#Double-precision_examples
    /// (Min. subnormal positive double)
    fn pos_min_subnormal() -> f64 {
        ((f64::MIN_EXP - (f64::MANTISSA_DIGITS as i32)) as f64).exp2()
    }

    #[test]
    fn float_pos_max_subnormal() {
        let value = pos_max_subnormal();
        assert!(value.is_subnormal());
        assert_eq!(value.to_bits(), 0x000F_FFFF_FFFF_FFFF_u64);
        ok(value);
    }

    #[test]
    fn float_pos_min_subnormal() {
        let value = pos_min_subnormal();
        assert!(value.is_subnormal());
        assert_eq!(value.to_bits(), 0x0000_0000_0000_0001_u64);
        assert_eq!(value * 0.5, 0.0);
        ok(value);
    }

    #[test]
    fn float_neg_max_subnormal() {
        let value = -pos_min_subnormal();
        assert!(value.is_subnormal());
        assert!(value.is_sign_negative());
        assert_eq!(value * 0.5, 0.0);
        err_negative(value);
    }

    #[test]
    fn float_neg_min_subnormal() {
        let value = -pos_max_subnormal();
        assert!(value.is_subnormal());
        assert!(value.is_sign_negative());
        err_negative(value);
    }

    #[test]
    fn float_pos_max_normal() {
        ok(f64::MAX);
    }

    #[test]
    fn float_pos_min_normal() {
        ok(f64::MIN_POSITIVE);
    }

    #[test]
    fn float_neg_max_normal() {
        err_negative(-f64::MIN_POSITIVE);
    }

    #[test]
    fn float_neg_min_normal() {
        err_negative(f64::MIN);
    }

    #[test]
    fn float_pos_normal() {
        ok(std::f64::consts::PI);
    }

    #[test]
    fn float_neg_normal() {
        err_negative(-std::f64::consts::PI);
    }

    fn ok(value: f64) {
        let pos_float = PositiveFiniteF64::try_from(value).unwrap();
        assert_eq!(pos_float.value(), value);
    }

    fn err_non_finite(value: f64) {
        let error = PositiveFiniteF64::try_from(value).unwrap_err();
        assert_eq!(error, InvalidFloatError::NonFinite);
    }

    fn err_negative(value: f64) {
        let error = PositiveFiniteF64::try_from(value).unwrap_err();
        assert_eq!(error, InvalidFloatError::Negative);
    }
}
