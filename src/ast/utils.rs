#[derive(Debug)]
pub struct PositiveFiniteF64 {
    value: f64,
}

#[derive(Debug, Eq, PartialEq)]
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
    use super::{PositiveFiniteF64, InvalidFloatError};

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
        ((f64::MIN_EXP - 1) as f64).exp2() * (1.0 - ((-(f64::MANTISSA_DIGITS as i32) + 1) as f64).exp2())
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
