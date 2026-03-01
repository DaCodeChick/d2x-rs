//! Binary I/O utilities for reading Descent file formats.
//!
//! This module provides extension traits for binary reading with consistent
//! error handling across all format parsers. Uses the `byteorder` crate
//! for efficient, zero-cost binary I/O operations.

use crate::error::Result;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Extension trait for reading binary data from cursors.
///
/// Provides consistent binary reading methods for all Descent format parsers.
/// All methods return `Result<T>` for uniform error handling.
///
/// This trait is implemented for `Cursor<&[u8]>` and `Cursor<&Vec<u8>>`.
pub trait ReadExt: Read {
    /// Read a single unsigned byte.
    fn read_u8(&mut self) -> Result<u8> {
        Ok(ReadBytesExt::read_u8(self)?)
    }

    /// Read a single signed byte.
    fn read_i8(&mut self) -> Result<i8> {
        Ok(ReadBytesExt::read_i8(self)?)
    }

    /// Read a 16-bit unsigned integer (little-endian).
    fn read_u16_le(&mut self) -> Result<u16> {
        Ok(self.read_u16::<LittleEndian>()?)
    }

    /// Read a 16-bit signed integer (little-endian).
    fn read_i16_le(&mut self) -> Result<i16> {
        Ok(self.read_i16::<LittleEndian>()?)
    }

    /// Read a 32-bit unsigned integer (little-endian).
    fn read_u32_le(&mut self) -> Result<u32> {
        Ok(self.read_u32::<LittleEndian>()?)
    }

    /// Read a 32-bit signed integer (little-endian).
    fn read_i32_le(&mut self) -> Result<i32> {
        Ok(self.read_i32::<LittleEndian>()?)
    }

    /// Read a 32-bit floating-point number (little-endian).
    fn read_f32_le(&mut self) -> Result<f32> {
        Ok(self.read_f32::<LittleEndian>()?)
    }

    /// Read a 16-bit unsigned integer (big-endian).
    fn read_u16_be(&mut self) -> Result<u16> {
        Ok(self.read_u16::<BigEndian>()?)
    }

    /// Read a 16-bit signed integer (big-endian).
    fn read_i16_be(&mut self) -> Result<i16> {
        Ok(self.read_i16::<BigEndian>()?)
    }

    /// Read a 32-bit unsigned integer (big-endian).
    fn read_u32_be(&mut self) -> Result<u32> {
        Ok(self.read_u32::<BigEndian>()?)
    }

    /// Read a 32-bit signed integer (big-endian).
    fn read_i32_be(&mut self) -> Result<i32> {
        Ok(self.read_i32::<BigEndian>()?)
    }

    /// Read a fixed number of bytes into a vector.
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Skip a fixed number of bytes.
    fn skip_bytes(&mut self, count: usize) -> Result<()>
    where
        Self: Seek,
    {
        self.seek(SeekFrom::Current(count as i64))?;
        Ok(())
    }
}

// Implement for Cursor<&[u8]>
impl ReadExt for Cursor<&[u8]> {}

// Implement for Cursor<&Vec<u8>>
impl ReadExt for Cursor<&Vec<u8>> {}

/// Read a null-terminated or null-padded string from a byte slice.
///
/// Stops at the first null byte or end of slice, whichever comes first.
/// Non-UTF8 sequences are replaced with the Unicode replacement character.
///
/// # Example
///
/// ```
/// # use descent_core::io::read_null_padded_string;
/// let data = b"PLAYER\0\0";
/// assert_eq!(read_null_padded_string(data), "PLAYER");
///
/// let data = b"TEST1234"; // No null terminator
/// assert_eq!(read_null_padded_string(data), "TEST1234");
/// ```
pub fn read_null_padded_string(data: &[u8]) -> String {
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42u8];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(ReadExt::read_u8(&mut cursor).unwrap(), 0x42);
    }

    #[test]
    fn test_read_i8() {
        let data = [0xFFu8]; // -1 as i8
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(ReadExt::read_i8(&mut cursor).unwrap(), -1);
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x34u8, 0x12]; // 0x1234 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u16_le().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_le() {
        let data = [0xFFu8, 0xFF]; // -1 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i16_le().unwrap(), -1);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78u8, 0x56, 0x34, 0x12]; // 0x12345678 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u32_le().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_le() {
        let data = [0xFFu8, 0xFF, 0xFF, 0xFF]; // -1 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i32_le().unwrap(), -1);
    }

    #[test]
    fn test_read_f32_le() {
        let data = [0x00u8, 0x00, 0x80, 0x3F]; // 1.0f32 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_f32_le().unwrap(), 1.0f32);
    }

    #[test]
    fn test_read_u16_be() {
        let data = [0x12u8, 0x34]; // 0x1234 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u16_be().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_be() {
        let data = [0xFFu8, 0xFF]; // -1 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i16_be().unwrap(), -1);
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x12u8, 0x34, 0x56, 0x78]; // 0x12345678 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u32_be().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_be() {
        let data = [0xFFu8, 0xFF, 0xFF, 0xFF]; // -1 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i32_be().unwrap(), -1);
    }

    #[test]
    fn test_read_bytes() {
        let data = [1u8, 2, 3, 4, 5];
        let mut cursor = Cursor::new(&data[..]);
        let result = cursor.read_bytes(3).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_skip_bytes() {
        let data = [1u8, 2, 3, 4, 5];
        let mut cursor = Cursor::new(&data[..]);
        cursor.skip_bytes(2).unwrap();
        assert_eq!(ReadExt::read_u8(&mut cursor).unwrap(), 3);
    }

    #[test]
    fn test_read_past_end() {
        let data = [1u8];
        let mut cursor = Cursor::new(&data[..]);
        assert!(cursor.read_u32_le().is_err());
    }

    #[test]
    fn test_read_null_padded_string() {
        // String with null terminator
        let data = b"HELLO\0\0\0";
        assert_eq!(read_null_padded_string(data), "HELLO");

        // String without null terminator
        let data = b"WORLD";
        assert_eq!(read_null_padded_string(data), "WORLD");

        // Empty string
        let data = b"\0\0\0";
        assert_eq!(read_null_padded_string(data), "");

        // String with embedded null
        let data = b"HEL\0LO";
        assert_eq!(read_null_padded_string(data), "HEL");
    }
}
