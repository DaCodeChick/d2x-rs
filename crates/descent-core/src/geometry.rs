//! Common geometric types and fixed-point vector operations.
//!
//! This module provides unified geometric types used across multiple parsers:
//! - `FixVector`: 3D vector with fixed-point coordinates
//! - `Uvl`: UV texture coordinates with lighting value
//!
//! These types are used in level geometry (level.rs) and 3D models (pof.rs).

use crate::fixed_point::Fix;

// ================================================================================================
// VECTOR TYPES
// ================================================================================================

/// 3D vector with fixed-point coordinates.
///
/// Used to represent positions, normals, and other 3D vectors in Descent's
/// fixed-point coordinate system.
///
/// # Examples
///
/// ```
/// use descent_core::geometry::FixVector;
/// use descent_core::fixed_point::Fix;
///
/// let vec = FixVector {
///     x: Fix::from(1.0),
///     y: Fix::from(2.5),
///     z: Fix::from(-3.0),
/// };
///
/// let [x, y, z] = vec.to_f32();
/// assert!((x - 1.0).abs() < 0.001);
/// assert!((y - 2.5).abs() < 0.001);
/// assert!((z + 3.0).abs() < 0.001);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixVector {
    pub x: Fix,
    pub y: Fix,
    pub z: Fix,
}

impl FixVector {
    /// Creates a new fixed-point vector from floating-point coordinates.
    ///
    /// # Examples
    ///
    /// ```
    /// use descent_core::geometry::FixVector;
    ///
    /// let vec = FixVector::from_f32(1.0, 2.5, -3.0);
    /// ```
    pub fn from_f32(x: f32, y: f32, z: f32) -> Self {
        Self {
            x: Fix::from(x),
            y: Fix::from(y),
            z: Fix::from(z),
        }
    }

    /// Creates a new fixed-point vector from fixed-point values.
    ///
    /// # Examples
    ///
    /// ```
    /// use descent_core::geometry::FixVector;
    /// use descent_core::fixed_point::Fix;
    ///
    /// let vec = FixVector::new(Fix::from(1.0), Fix::from(2.0), Fix::from(3.0));
    /// ```
    pub const fn new(x: Fix, y: Fix, z: Fix) -> Self {
        Self { x, y, z }
    }

    /// Zero vector (0, 0, 0).
    pub const ZERO: Self = Self {
        x: Fix::ZERO,
        y: Fix::ZERO,
        z: Fix::ZERO,
    };

    /// Unit X vector (1, 0, 0).
    pub const UNIT_X: Self = Self {
        x: Fix::ONE,
        y: Fix::ZERO,
        z: Fix::ZERO,
    };

    /// Unit Y vector (0, 1, 0).
    pub const UNIT_Y: Self = Self {
        x: Fix::ZERO,
        y: Fix::ONE,
        z: Fix::ZERO,
    };

    /// Unit Z vector (0, 0, 1).
    pub const UNIT_Z: Self = Self {
        x: Fix::ZERO,
        y: Fix::ZERO,
        z: Fix::ONE,
    };

    /// Converts fixed-point vector to floating-point array [x, y, z].
    ///
    /// # Examples
    ///
    /// ```
    /// use descent_core::geometry::FixVector;
    ///
    /// let vec = FixVector::from_f32(1.0, 2.0, 3.0);
    /// let [x, y, z] = vec.to_f32();
    /// ```
    pub fn to_f32(self) -> [f32; 3] {
        [self.x.into(), self.y.into(), self.z.into()]
    }

    /// Converts fixed-point vector to floating-point array [x, y, z] (alias for glam compatibility).
    ///
    /// This method provides compatibility with code that expects `to_vec3()` method name.
    pub fn to_vec3(self) -> [f32; 3] {
        self.to_f32()
    }

    /// Calculates the dot product of two vectors.
    pub fn dot(self, other: Self) -> Fix {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    /// Calculates the squared length of the vector (avoids square root).
    pub fn length_squared(self) -> Fix {
        self.dot(self)
    }
}

impl Default for FixVector {
    fn default() -> Self {
        Self::ZERO
    }
}

impl From<(Fix, Fix, Fix)> for FixVector {
    fn from((x, y, z): (Fix, Fix, Fix)) -> Self {
        Self { x, y, z }
    }
}

impl From<FixVector> for (Fix, Fix, Fix) {
    fn from(vec: FixVector) -> Self {
        (vec.x, vec.y, vec.z)
    }
}

impl From<(f32, f32, f32)> for FixVector {
    fn from((x, y, z): (f32, f32, f32)) -> Self {
        Self::from_f32(x, y, z)
    }
}

impl From<FixVector> for [f32; 3] {
    fn from(vec: FixVector) -> Self {
        vec.to_f32()
    }
}

// ================================================================================================
// UV COORDINATES
// ================================================================================================

/// UV texture coordinates with lighting value.
///
/// Used for texture-mapped polygons in both level geometry and 3D models.
/// The `l` component is a light value that modulates the texture brightness.
///
/// # Examples
///
/// ```
/// use descent_core::geometry::Uvl;
/// use descent_core::fixed_point::Fix;
///
/// let uvl = Uvl {
///     u: Fix::from(0.5),
///     v: Fix::from(0.5),
///     l: Fix::from(1.0),
/// };
///
/// let [u, v, l] = uvl.to_f32();
/// assert!((u - 0.5).abs() < 0.001);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Uvl {
    /// U texture coordinate (horizontal).
    pub u: Fix,
    /// V texture coordinate (vertical).
    pub v: Fix,
    /// Light value (brightness multiplier).
    pub l: Fix,
}

impl Uvl {
    /// Creates a new UVL from floating-point values.
    ///
    /// # Examples
    ///
    /// ```
    /// use descent_core::geometry::Uvl;
    ///
    /// let uvl = Uvl::from_f32(0.5, 0.5, 1.0);
    /// ```
    pub fn from_f32(u: f32, v: f32, l: f32) -> Self {
        Self {
            u: Fix::from(u),
            v: Fix::from(v),
            l: Fix::from(l),
        }
    }

    /// Creates a new UVL from fixed-point values.
    pub const fn new(u: Fix, v: Fix, l: Fix) -> Self {
        Self { u, v, l }
    }

    /// Converts to floating-point array [u, v, l].
    ///
    /// # Examples
    ///
    /// ```
    /// use descent_core::geometry::Uvl;
    ///
    /// let uvl = Uvl::from_f32(0.5, 0.5, 1.0);
    /// let [u, v, l] = uvl.to_f32();
    /// ```
    pub fn to_f32(self) -> [f32; 3] {
        [self.u.into(), self.v.into(), self.l.into()]
    }
}

impl Default for Uvl {
    fn default() -> Self {
        Self {
            u: Fix::ZERO,
            v: Fix::ZERO,
            l: Fix::ONE,
        }
    }
}

impl From<(Fix, Fix, Fix)> for Uvl {
    fn from((u, v, l): (Fix, Fix, Fix)) -> Self {
        Self { u, v, l }
    }
}

impl From<Uvl> for (Fix, Fix, Fix) {
    fn from(uvl: Uvl) -> Self {
        (uvl.u, uvl.v, uvl.l)
    }
}

impl From<(f32, f32, f32)> for Uvl {
    fn from((u, v, l): (f32, f32, f32)) -> Self {
        Self::from_f32(u, v, l)
    }
}

impl From<Uvl> for [f32; 3] {
    fn from(uvl: Uvl) -> Self {
        uvl.to_f32()
    }
}

// ================================================================================================
// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fix_vector_new() {
        let vec = FixVector::new(Fix::from(1.0), Fix::from(2.0), Fix::from(3.0));
        let [x, y, z] = vec.to_f32();
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 2.0).abs() < 0.001);
        assert!((z - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_fix_vector_from_f32() {
        let vec = FixVector::from_f32(1.5, -2.5, 3.0);
        let [x, y, z] = vec.to_f32();
        assert!((x - 1.5).abs() < 0.001);
        assert!((y + 2.5).abs() < 0.001);
        assert!((z - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_fix_vector_constants() {
        assert_eq!(FixVector::ZERO.to_f32(), [0.0, 0.0, 0.0]);
        assert_eq!(FixVector::UNIT_X.to_f32(), [1.0, 0.0, 0.0]);
        assert_eq!(FixVector::UNIT_Y.to_f32(), [0.0, 1.0, 0.0]);
        assert_eq!(FixVector::UNIT_Z.to_f32(), [0.0, 0.0, 1.0]);
    }

    #[test]
    fn test_fix_vector_to_vec3() {
        let vec = FixVector::from_f32(1.0, 2.0, 3.0);
        let arr = vec.to_vec3();
        assert_eq!(arr, vec.to_f32());
    }

    #[test]
    fn test_fix_vector_dot() {
        let v1 = FixVector::from_f32(1.0, 2.0, 3.0);
        let v2 = FixVector::from_f32(4.0, 5.0, 6.0);
        let dot: f32 = v1.dot(v2).into();
        // 1*4 + 2*5 + 3*6 = 4 + 10 + 18 = 32
        assert!((dot - 32.0).abs() < 0.1);
    }

    #[test]
    fn test_fix_vector_length_squared() {
        let vec = FixVector::from_f32(3.0, 4.0, 0.0);
        let len_sq: f32 = vec.length_squared().into();
        // 3^2 + 4^2 = 9 + 16 = 25
        assert!((len_sq - 25.0).abs() < 0.1);
    }

    #[test]
    fn test_fix_vector_from_tuple() {
        let vec: FixVector = (Fix::from(1.0), Fix::from(2.0), Fix::from(3.0)).into();
        let [x, y, z] = vec.to_f32();
        assert!((x - 1.0).abs() < 0.001);
        assert!((y - 2.0).abs() < 0.001);
        assert!((z - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_fix_vector_from_f32_tuple() {
        let vec: FixVector = (1.5_f32, 2.5_f32, 3.5_f32).into();
        let [x, y, z] = vec.to_f32();
        assert!((x - 1.5).abs() < 0.001);
        assert!((y - 2.5).abs() < 0.001);
        assert!((z - 3.5).abs() < 0.001);
    }

    #[test]
    fn test_fix_vector_into_array() {
        let vec = FixVector::from_f32(1.0, 2.0, 3.0);
        let arr: [f32; 3] = vec.into();
        assert!((arr[0] - 1.0).abs() < 0.001);
        assert!((arr[1] - 2.0).abs() < 0.001);
        assert!((arr[2] - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_uvl_new() {
        let uvl = Uvl::new(Fix::from(0.5), Fix::from(0.75), Fix::from(1.0));
        let [u, v, l] = uvl.to_f32();
        assert!((u - 0.5).abs() < 0.001);
        assert!((v - 0.75).abs() < 0.001);
        assert!((l - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_uvl_from_f32() {
        let uvl = Uvl::from_f32(0.25, 0.5, 0.75);
        let [u, v, l] = uvl.to_f32();
        assert!((u - 0.25).abs() < 0.001);
        assert!((v - 0.5).abs() < 0.001);
        assert!((l - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_uvl_default() {
        let uvl = Uvl::default();
        let [u, v, l] = uvl.to_f32();
        assert!((u - 0.0).abs() < 0.001);
        assert!((v - 0.0).abs() < 0.001);
        assert!((l - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_uvl_from_tuple() {
        let uvl: Uvl = (Fix::from(0.5), Fix::from(0.5), Fix::from(1.0)).into();
        let [u, v, l] = uvl.to_f32();
        assert!((u - 0.5).abs() < 0.001);
        assert!((v - 0.5).abs() < 0.001);
        assert!((l - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_uvl_from_f32_tuple() {
        let uvl: Uvl = (0.25_f32, 0.5_f32, 0.75_f32).into();
        let [u, v, l] = uvl.to_f32();
        assert!((u - 0.25).abs() < 0.001);
        assert!((v - 0.5).abs() < 0.001);
        assert!((l - 0.75).abs() < 0.001);
    }

    #[test]
    fn test_uvl_into_array() {
        let uvl = Uvl::from_f32(0.5, 0.5, 1.0);
        let arr: [f32; 3] = uvl.into();
        assert!((arr[0] - 0.5).abs() < 0.001);
        assert!((arr[1] - 0.5).abs() < 0.001);
        assert!((arr[2] - 1.0).abs() < 0.001);
    }
}
