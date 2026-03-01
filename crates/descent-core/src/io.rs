//! Binary I/O utilities for reading Descent file formats.
//!
//! This module provides utility functions and re-exports for binary I/O operations.
//! Uses the `byteorder` crate for efficient, zero-cost binary reading.
//!
//! # Example
//!
//! ```
//! use std::io::Cursor;
//! use byteorder::{ReadBytesExt, LittleEndian};
//!
//! let data = [0x78, 0x56, 0x34, 0x12];
//! let mut cursor = Cursor::new(&data[..]);
//! let value = cursor.read_u32::<LittleEndian>().unwrap();
//! assert_eq!(value, 0x12345678);
//! ```

use crate::error::Result;
use std::io::{Read, Seek, SeekFrom};

// Re-export byteorder types for convenience
pub use byteorder::{BigEndian, LittleEndian, ReadBytesExt};

/// Read a fixed number of bytes into a vector.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
/// use descent_core::io::read_bytes;
///
/// let data = [1, 2, 3, 4, 5];
/// let mut cursor = Cursor::new(&data[..]);
/// let result = read_bytes(&mut cursor, 3).unwrap();
/// assert_eq!(result, vec![1, 2, 3]);
/// ```
pub fn read_bytes<R: Read>(reader: &mut R, count: usize) -> Result<Vec<u8>> {
    let mut buf = vec![0u8; count];
    reader.read_exact(&mut buf)?;
    Ok(buf)
}

/// Skip a fixed number of bytes.
///
/// # Example
///
/// ```
/// use std::io::Cursor;
/// use byteorder::ReadBytesExt;
/// use descent_core::io::skip_bytes;
///
/// let data = [1, 2, 3, 4, 5];
/// let mut cursor = Cursor::new(&data[..]);
/// skip_bytes(&mut cursor, 2).unwrap();
/// assert_eq!(cursor.read_u8().unwrap(), 3);
/// ```
pub fn skip_bytes<R: Read + Seek>(reader: &mut R, count: usize) -> Result<()> {
    reader.seek(SeekFrom::Current(count as i64))?;
    Ok(())
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
    let end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    String::from_utf8_lossy(&data[..end]).into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_read_u8() {
        let data = [0x42u8];
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u8().unwrap(), 0x42);
    }

    #[test]
    fn test_read_i8() {
        let data = [0xFFu8]; // -1 as i8
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i8().unwrap(), -1);
    }

    #[test]
    fn test_read_u16_le() {
        let data = [0x34u8, 0x12]; // 0x1234 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u16::<LittleEndian>().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_le() {
        let data = [0xFFu8, 0xFF]; // -1 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i16::<LittleEndian>().unwrap(), -1);
    }

    #[test]
    fn test_read_u32_le() {
        let data = [0x78u8, 0x56, 0x34, 0x12]; // 0x12345678 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u32::<LittleEndian>().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_le() {
        let data = [0xFFu8, 0xFF, 0xFF, 0xFF]; // -1 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i32::<LittleEndian>().unwrap(), -1);
    }

    #[test]
    fn test_read_f32_le() {
        let data = [0x00u8, 0x00, 0x80, 0x3F]; // 1.0f32 in little-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_f32::<LittleEndian>().unwrap(), 1.0f32);
    }

    #[test]
    fn test_read_u16_be() {
        let data = [0x12u8, 0x34]; // 0x1234 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u16::<BigEndian>().unwrap(), 0x1234);
    }

    #[test]
    fn test_read_i16_be() {
        let data = [0xFFu8, 0xFF]; // -1 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i16::<BigEndian>().unwrap(), -1);
    }

    #[test]
    fn test_read_u32_be() {
        let data = [0x12u8, 0x34, 0x56, 0x78]; // 0x12345678 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_u32::<BigEndian>().unwrap(), 0x12345678);
    }

    #[test]
    fn test_read_i32_be() {
        let data = [0xFFu8, 0xFF, 0xFF, 0xFF]; // -1 in big-endian
        let mut cursor = Cursor::new(&data[..]);
        assert_eq!(cursor.read_i32::<BigEndian>().unwrap(), -1);
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
