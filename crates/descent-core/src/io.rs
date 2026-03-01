//! Binary I/O utilities for reading Descent file formats.
//!
//! This module provides extension traits for binary reading with consistent
//! error handling across all format parsers.

use crate::error::Result;
use std::io::{Cursor, Read, Seek, SeekFrom};

/// Extension trait for reading binary data from cursors.
///
/// Provides consistent little-endian reading methods for all Descent format parsers.
/// All methods return `Result<T>` for uniform error handling.
pub trait ReadExt {
    /// Read a single unsigned byte.
    fn read_u8(&mut self) -> Result<u8>;

    /// Read a single signed byte.
    fn read_i8(&mut self) -> Result<i8>;

    /// Read a 16-bit unsigned integer (little-endian).
    fn read_u16_le(&mut self) -> Result<u16>;

    /// Read a 16-bit signed integer (little-endian).
    fn read_i16_le(&mut self) -> Result<i16>;

    /// Read a 32-bit unsigned integer (little-endian).
    fn read_u32_le(&mut self) -> Result<u32>;

    /// Read a 32-bit signed integer (little-endian).
    fn read_i32_le(&mut self) -> Result<i32>;

    /// Read a 32-bit floating-point number (little-endian).
    fn read_f32_le(&mut self) -> Result<f32>;

    /// Read a 16-bit unsigned integer (big-endian).
    fn read_u16_be(&mut self) -> Result<u16>;

    /// Read a 16-bit signed integer (big-endian).
    fn read_i16_be(&mut self) -> Result<i16>;

    /// Read a 32-bit unsigned integer (big-endian).
    fn read_u32_be(&mut self) -> Result<u32>;

    /// Read a 32-bit signed integer (big-endian).
    fn read_i32_be(&mut self) -> Result<i32>;

    /// Read a fixed number of bytes into a vector.
    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>>;

    /// Skip a fixed number of bytes.
    fn skip_bytes(&mut self, count: usize) -> Result<()>;
}

impl ReadExt for Cursor<&[u8]> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_i16_le(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i32_le(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn read_u16_be(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_i16_be(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_u32_be(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_i32_be(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn skip_bytes(&mut self, count: usize) -> Result<()> {
        self.seek(SeekFrom::Current(count as i64))?;
        Ok(())
    }
}

impl ReadExt for Cursor<&Vec<u8>> {
    fn read_u8(&mut self) -> Result<u8> {
        let mut buf = [0u8; 1];
        self.read_exact(&mut buf)?;
        Ok(buf[0])
    }

    fn read_i8(&mut self) -> Result<i8> {
        Ok(self.read_u8()? as i8)
    }

    fn read_u16_le(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_le_bytes(buf))
    }

    fn read_i16_le(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_le_bytes(buf))
    }

    fn read_u32_le(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_le_bytes(buf))
    }

    fn read_i32_le(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_le_bytes(buf))
    }

    fn read_f32_le(&mut self) -> Result<f32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(f32::from_le_bytes(buf))
    }

    fn read_u16_be(&mut self) -> Result<u16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(u16::from_be_bytes(buf))
    }

    fn read_i16_be(&mut self) -> Result<i16> {
        let mut buf = [0u8; 2];
        self.read_exact(&mut buf)?;
        Ok(i16::from_be_bytes(buf))
    }

    fn read_u32_be(&mut self) -> Result<u32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(u32::from_be_bytes(buf))
    }

    fn read_i32_be(&mut self) -> Result<i32> {
        let mut buf = [0u8; 4];
        self.read_exact(&mut buf)?;
        Ok(i32::from_be_bytes(buf))
    }

    fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let mut buf = vec![0u8; count];
        self.read_exact(&mut buf)?;
        Ok(buf)
    }

    fn skip_bytes(&mut self, count: usize) -> Result<()> {
        self.seek(SeekFrom::Current(count as i64))?;
        Ok(())
    }
}

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
    let null_pos = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..null_pos]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read_u8() {
        let data = [0x42];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u8().unwrap(), 0x42);
    }

    #[test]
    fn test_read_i8() {
        let data = [0xFF]; // -1 in signed
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i8().unwrap(), -1);
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x34, 0x12]; // 0x1234 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u16_le().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_le() {
        let data = [0xFF, 0xFF]; // -1 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i16_le().unwrap(), -1);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78, 0x56, 0x34, 0x12]; // 0x12345678 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u32_le().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_le() {
        let data = [0xFF, 0xFF, 0xFF, 0xFF]; // -1 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i32_le().unwrap(), -1);
    }

    #[test]
    fn test_read_f32_le() {
        let value = 3.14159f32;
        let data = value.to_le_bytes();
        let mut cursor = Cursor::new(&data[..]);
        let result = cursor.read_f32_le().unwrap();
        assert!((result - value).abs() < 0.0001);
    }

    #[test]
    fn test_read_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(&data[..]);
        let result = cursor.read_bytes(3).unwrap();
        assert_eq!(result, vec![1, 2, 3]);
    }

    #[test]
    fn test_skip_bytes() {
        let data = [1, 2, 3, 4, 5];
        let mut cursor = Cursor::new(&data[..]);
        cursor.skip_bytes(2).unwrap();
        assert_eq!(cursor.read_u8().unwrap(), 3);
    }

    #[test]
    fn test_read_null_padded_string() {
        assert_eq!(read_null_padded_string(b"PLAYER\0\0"), "PLAYER");
        assert_eq!(read_null_padded_string(b"TEST1234"), "TEST1234");
        assert_eq!(read_null_padded_string(b"HI\0THERE"), "HI");
        assert_eq!(read_null_padded_string(b""), "");
        assert_eq!(read_null_padded_string(b"\0\0\0\0"), "");
    }

    #[test]
    fn test_read_past_end() {
        let data = [1, 2, 3];
        let mut cursor = Cursor::new(&data[..]);
        cursor.skip_bytes(2).unwrap();
        // Try to read 2 bytes when only 1 is left
        assert!(cursor.read_u16_le().is_err());
    }
}
