//! Fixed-point arithmetic types used throughout Descent file formats.
//!
//! Descent uses 16.16 fixed-point numbers (16 bits integer, 16 bits fraction)
//! for representing positions, velocities, and other game values.

use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};

/// Fixed-point multiplier for 16.16 format (2^16 = 65536)
pub const I2X_MULTIPLIER: i32 = 65536;

/// Fixed-point number in 16.16 format.
///
/// The value is stored as an `i32` where:
/// - Upper 16 bits: integer part
/// - Lower 16 bits: fractional part
///
/// # Examples
///
/// ```
/// # use descent_core::fixed_point::Fix;
/// let one = Fix::ONE;
/// let half = Fix::from(0.5);
/// let result = one + half;
/// assert!((f32::from(result) - 1.5).abs() < 0.0001);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Fix(pub i32);

impl Fix {
    /// Zero (0.0)
    pub const ZERO: Self = Self(0);

    /// One (1.0)
    pub const ONE: Self = Self(I2X_MULTIPLIER);

    /// Get the raw representation of the fixed-point value.
    ///
    /// Use this when you need direct access to the internal i32 value.
    pub const fn to_raw(self) -> i32 {
        self.0
    }

    /// Get the absolute value.
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }

    /// Get the sign: -1 for negative, 0 for zero, 1 for positive.
    pub fn signum(self) -> i32 {
        self.0.signum()
    }
}

// ================================================================================================
// CONVERSIONS
// ================================================================================================

impl From<i32> for Fix {
    /// Create a fixed-point value from raw i32 representation.
    ///
    /// Use this when you have a raw fixed-point value that's already in 16.16 format.
    ///
    /// # Example
    ///
    /// ```
    /// # use descent_core::fixed_point::Fix;
    /// let fix = Fix::from(65536_i32); // 1.0
    /// let f: f32 = fix.into();
    /// assert_eq!(f, 1.0);
    /// ```
    fn from(raw: i32) -> Self {
        Self(raw)
    }
}

impl From<Fix> for i32 {
    /// Get the raw i32 representation of the fixed-point value.
    fn from(fix: Fix) -> Self {
        fix.0
    }
}

impl From<f32> for Fix {
    /// Create a fixed-point value from f32.
    fn from(val: f32) -> Self {
        Self((val * I2X_MULTIPLIER as f32) as i32)
    }
}

impl From<f64> for Fix {
    /// Create a fixed-point value from f64.
    fn from(val: f64) -> Self {
        Self((val * I2X_MULTIPLIER as f64) as i32)
    }
}

impl From<Fix> for f32 {
    /// Convert fixed-point to f32.
    fn from(fix: Fix) -> Self {
        fix.0 as f32 / I2X_MULTIPLIER as f32
    }
}

impl From<Fix> for f64 {
    /// Convert fixed-point to f64.
    fn from(fix: Fix) -> Self {
        fix.0 as f64 / I2X_MULTIPLIER as f64
    }
}

impl Add for Fix {
    type Output = Self;

    fn add(self, rhs: Self) -> Self {
        Self(self.0 + rhs.0)
    }
}

impl AddAssign for Fix {
    fn add_assign(&mut self, rhs: Self) {
        self.0 += rhs.0;
    }
}

impl Sub for Fix {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self {
        Self(self.0 - rhs.0)
    }
}

impl SubAssign for Fix {
    fn sub_assign(&mut self, rhs: Self) {
        self.0 -= rhs.0;
    }
}

impl Mul for Fix {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        // Multiply and shift right to maintain precision
        let result = (self.0 as i64 * rhs.0 as i64) >> 16;
        Self(result as i32)
    }
}

impl Div for Fix {
    type Output = Self;

    fn div(self, rhs: Self) -> Self {
        // Shift left before division to maintain precision
        let result = ((self.0 as i64) << 16) / rhs.0 as i64;
        Self(result as i32)
    }
}

impl Neg for Fix {
    type Output = Self;

    fn neg(self) -> Self {
        Self(-self.0)
    }
}

impl std::fmt::Display for Fix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let value: f32 = (*self).into();
        write!(f, "{}", value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constants() {
        assert_eq!(Fix::ZERO.0, 0);
        assert_eq!(Fix::ONE.0, I2X_MULTIPLIER);
    }

    #[test]
    fn test_from_f32() {
        let fix = Fix::from(1.0);
        assert_eq!(fix.0, I2X_MULTIPLIER);

        let fix = Fix::from(2.5);
        let expected = (2.5 * I2X_MULTIPLIER as f32) as i32;
        assert_eq!(fix.0, expected);
    }

    #[test]
    fn test_to_f32() {
        let fix = Fix(I2X_MULTIPLIER);
        let f: f32 = fix.into();
        assert_eq!(f, 1.0);

        let fix = Fix::from(2.5);
        let f: f32 = fix.into();
        assert!((f - 2.5).abs() < 0.0001);
    }

    #[test]
    fn test_roundtrip() {
        let values = [0.0, 1.0, -1.0, 2.5, -3.75, 100.5];
        for &val in &values {
            let fix = Fix::from(val);
            let result: f32 = fix.into();
            assert!((result - val).abs() < 0.0001, "Failed for {}", val);
        }
    }

    #[test]
    fn test_addition() {
        let a = Fix::from(1.5);
        let b = Fix::from(2.5);
        let c = a + b;
        let result: f32 = c.into();
        assert!((result - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_subtraction() {
        let a = Fix::from(5.5);
        let b = Fix::from(2.5);
        let c = a - b;
        let result: f32 = c.into();
        assert!((result - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_multiplication() {
        let a = Fix::from(2.0);
        let b = Fix::from(3.0);
        let c = a * b;
        let result: f32 = c.into();
        assert!((result - 6.0).abs() < 0.0001);

        let a = Fix::from(1.5);
        let b = Fix::from(2.0);
        let c = a * b;
        let result: f32 = c.into();
        assert!((result - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_division() {
        let a = Fix::from(6.0);
        let b = Fix::from(2.0);
        let c = a / b;
        let result: f32 = c.into();
        assert!((result - 3.0).abs() < 0.0001);

        let a = Fix::from(5.0);
        let b = Fix::from(2.0);
        let c = a / b;
        let result: f32 = c.into();
        assert!((result - 2.5).abs() < 0.0001);
    }

    #[test]
    fn test_negation() {
        let a = Fix::from(3.5);
        let b = -a;
        let result: f32 = b.into();
        assert!((result - -3.5).abs() < 0.0001);
    }

    #[test]
    fn test_abs() {
        let a = Fix::from(-3.5);
        let b = a.abs();
        let result: f32 = b.into();
        assert!((result - 3.5).abs() < 0.0001);

        let a = Fix::from(3.5);
        let b = a.abs();
        let result: f32 = b.into();
        assert!((result - 3.5).abs() < 0.0001);
    }

    #[test]
    fn test_signum() {
        assert_eq!(Fix::from(5.0).signum(), 1);
        assert_eq!(Fix::from(-5.0).signum(), -1);
        assert_eq!(Fix::ZERO.signum(), 0);
    }

    #[test]
    fn test_add_assign() {
        let mut a = Fix::from(1.5);
        a += Fix::from(2.5);
        let result: f32 = a.into();
        assert!((result - 4.0).abs() < 0.0001);
    }

    #[test]
    fn test_sub_assign() {
        let mut a = Fix::from(5.5);
        a -= Fix::from(2.5);
        let result: f32 = a.into();
        assert!((result - 3.0).abs() < 0.0001);
    }

    #[test]
    fn test_comparison() {
        let a = Fix::from(1.0);
        let b = Fix::from(2.0);
        assert!(a < b);
        assert!(b > a);
        assert!(a == Fix::ONE);
    }

    #[test]
    fn test_display() {
        let fix = Fix::from(3.14159);
        let s = format!("{}", fix);
        assert!(s.starts_with("3.14"));
    }
}
