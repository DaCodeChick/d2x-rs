//! Common validation helpers for parsers.
//!
//! This module provides reusable validation functions to reduce code duplication
//! and improve error messages across all parsers.

use crate::error::{AssetError, Result};

// ================================================================================================
// SIGNATURE VALIDATION
// ================================================================================================

/// Validates a 4-byte signature matches the expected value.
///
/// # Arguments
///
/// * `actual` - The actual signature read from the file
/// * `expected` - The expected signature value
/// * `format_name` - Name of the format (e.g., "HAM", "HOG", "POF")
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_signature;
///
/// let result = validate_signature(0x21474F48, 0x21474F48, "HOG");
/// assert!(result.is_ok());
///
/// let result = validate_signature(0x12345678, 0x21474F48, "HOG");
/// assert!(result.is_err());
/// ```
pub fn validate_signature(actual: u32, expected: u32, format_name: &str) -> Result<()> {
    if actual != expected {
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {} signature: expected 0x{:08X}, got 0x{:08X}",
            format_name, expected, actual
        )));
    }
    Ok(())
}

/// Validates a string signature matches the expected value.
///
/// # Arguments
///
/// * `actual` - The actual signature string read from the file
/// * `expected` - The expected signature string
/// * `format_name` - Name of the format (e.g., "HMP", "MIDI")
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_string_signature;
///
/// let result = validate_string_signature("HMIMIDIP", "HMIMIDIP", "HMP");
/// assert!(result.is_ok());
///
/// let result = validate_string_signature("INVALID", "HMIMIDIP", "HMP");
/// assert!(result.is_err());
/// ```
pub fn validate_string_signature(actual: &str, expected: &str, format_name: &str) -> Result<()> {
    if actual != expected {
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {} signature: expected '{}', got '{}'",
            format_name, expected, actual
        )));
    }
    Ok(())
}

// ================================================================================================
// RANGE VALIDATION
// ================================================================================================

/// Validates a value is within an inclusive range.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_range;
///
/// let result = validate_range(5, 1, 10, "count");
/// assert!(result.is_ok());
///
/// let result = validate_range(15, 1, 10, "count");
/// assert!(result.is_err());
/// ```
pub fn validate_range<T>(value: T, min: T, max: T, field_name: &str) -> Result<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min || value > max {
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {}: {} (must be between {} and {})",
            field_name, value, min, max
        )));
    }
    Ok(())
}

/// Validates a value does not exceed a maximum.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_max;
///
/// let result = validate_max(5, 10, "texture count", "MAX_TEXTURES");
/// assert!(result.is_ok());
///
/// let result = validate_max(15, 10, "texture count", "MAX_TEXTURES");
/// assert!(result.is_err());
/// ```
pub fn validate_max<T>(value: T, max: T, field_name: &str, max_name: &str) -> Result<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value > max {
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {}: {} (must be <= {} = {})",
            field_name, value, max_name, max
        )));
    }
    Ok(())
}

/// Validates a value is at least a minimum.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_min;
///
/// let result = validate_min(5, 1, "track count");
/// assert!(result.is_ok());
///
/// let result = validate_min(0, 1, "track count");
/// assert!(result.is_err());
/// ```
pub fn validate_min<T>(value: T, min: T, field_name: &str) -> Result<()>
where
    T: PartialOrd + std::fmt::Display,
{
    if value < min {
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {}: {} (must be >= {})",
            field_name, value, min
        )));
    }
    Ok(())
}

/// Validates a count is non-zero.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_non_zero;
///
/// let result = validate_non_zero(5, "vertex count");
/// assert!(result.is_ok());
///
/// let result = validate_non_zero(0, "vertex count");
/// assert!(result.is_err());
/// ```
pub fn validate_non_zero<T>(value: T, field_name: &str) -> Result<()>
where
    T: PartialEq + Default + std::fmt::Display,
{
    if value == T::default() {
        return Err(AssetError::InvalidFormat(format!(
            "{} cannot be zero",
            field_name
        )));
    }
    Ok(())
}

// ================================================================================================
// VERSION VALIDATION
// ================================================================================================

/// Validates a version is one of the expected values.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_version;
///
/// let result = validate_version(2, &[2, 3], "HAM");
/// assert!(result.is_ok());
///
/// let result = validate_version(5, &[2, 3], "HAM");
/// assert!(result.is_err());
/// ```
pub fn validate_version<T>(actual: T, expected: &[T], format_name: &str) -> Result<()>
where
    T: PartialEq + std::fmt::Display + Copy,
{
    if !expected.contains(&actual) {
        let expected_str = expected
            .iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(" or ");
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {} version: expected {}, got {}",
            format_name, expected_str, actual
        )));
    }
    Ok(())
}

// ================================================================================================
// DATA VALIDATION
// ================================================================================================

/// Validates a buffer has sufficient remaining bytes.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_buffer_size;
///
/// let data = vec![1, 2, 3, 4, 5];
/// let result = validate_buffer_size(data.len(), 10, 3);
/// assert!(result.is_ok());
///
/// let result = validate_buffer_size(data.len(), 10, 15);
/// assert!(result.is_err());
/// ```
pub fn validate_buffer_size(
    current_pos: usize,
    buffer_len: usize,
    required_bytes: usize,
) -> Result<()> {
    let remaining = buffer_len.saturating_sub(current_pos);
    if remaining < required_bytes {
        return Err(AssetError::InvalidFormat(format!(
            "Unexpected end of data: need {} bytes, but only {} remaining",
            required_bytes, remaining
        )));
    }
    Ok(())
}

/// Validates an index is within bounds for a collection.
///
/// # Examples
///
/// ```
/// use descent_core::validation::validate_index;
///
/// let result = validate_index(5, 10, "vertex index");
/// assert!(result.is_ok());
///
/// let result = validate_index(15, 10, "vertex index");
/// assert!(result.is_err());
/// ```
pub fn validate_index(index: usize, max: usize, field_name: &str) -> Result<()> {
    if index >= max {
        return Err(AssetError::InvalidFormat(format!(
            "Invalid {}: {} (must be < {})",
            field_name, index, max
        )));
    }
    Ok(())
}

// ================================================================================================
// TESTS
// ================================================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // Signature validation tests
    #[test]
    fn test_validate_signature_success() {
        assert!(validate_signature(0x21474F48, 0x21474F48, "HOG").is_ok());
    }

    #[test]
    fn test_validate_signature_failure() {
        let result = validate_signature(0x12345678, 0x21474F48, "HOG");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid HOG signature"));
    }

    #[test]
    fn test_validate_string_signature_success() {
        assert!(validate_string_signature("HMIMIDIP", "HMIMIDIP", "HMP").is_ok());
    }

    #[test]
    fn test_validate_string_signature_failure() {
        let result = validate_string_signature("INVALID", "HMIMIDIP", "HMP");
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Invalid HMP signature"));
    }

    // Range validation tests
    #[test]
    fn test_validate_range_success() {
        assert!(validate_range(5, 1, 10, "count").is_ok());
        assert!(validate_range(1, 1, 10, "count").is_ok());
        assert!(validate_range(10, 1, 10, "count").is_ok());
    }

    #[test]
    fn test_validate_range_failure() {
        assert!(validate_range(0, 1, 10, "count").is_err());
        assert!(validate_range(11, 1, 10, "count").is_err());
    }

    #[test]
    fn test_validate_max_success() {
        assert!(validate_max(5, 10, "texture count", "MAX_TEXTURES").is_ok());
    }

    #[test]
    fn test_validate_max_failure() {
        let result = validate_max(15, 10, "texture count", "MAX_TEXTURES");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("MAX_TEXTURES"));
    }

    #[test]
    fn test_validate_min_success() {
        assert!(validate_min(5, 1, "track count").is_ok());
    }

    #[test]
    fn test_validate_min_failure() {
        let result = validate_min(0, 1, "track count");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be >= 1"));
    }

    #[test]
    fn test_validate_non_zero_success() {
        assert!(validate_non_zero(5, "vertex count").is_ok());
    }

    #[test]
    fn test_validate_non_zero_failure() {
        let result = validate_non_zero(0, "vertex count");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be zero"));
    }

    // Version validation tests
    #[test]
    fn test_validate_version_success() {
        assert!(validate_version(2, &[2, 3], "HAM").is_ok());
        assert!(validate_version(3, &[2, 3], "HAM").is_ok());
    }

    #[test]
    fn test_validate_version_failure() {
        let result = validate_version(5, &[2, 3], "HAM");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("expected 2 or 3"));
    }

    // Data validation tests
    #[test]
    fn test_validate_buffer_size_success() {
        assert!(validate_buffer_size(5, 100, 10).is_ok());
    }

    #[test]
    fn test_validate_buffer_size_failure() {
        let result = validate_buffer_size(95, 100, 10);
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("Unexpected end of data"));
    }

    #[test]
    fn test_validate_index_success() {
        assert!(validate_index(5, 10, "vertex index").is_ok());
    }

    #[test]
    fn test_validate_index_failure() {
        let result = validate_index(15, 10, "vertex index");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("must be < 10"));
    }
}
