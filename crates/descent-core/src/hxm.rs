//! HXM (HaMster eXtended/Mod) file format parser
//!
//! HXM files contain custom robot and model data that override or extend HAM file definitions.
//! They are used in Descent 2 Vertigo expansion and D2X-XL custom content.
//!
//! **File Format**:
//! - Signature: `HMX!` (0x21584D48)
//! - Version: 1
//! - Endianness: Little-endian
//!
//! **Structure**:
//! ```text
//! Offset  Size  Description
//! ------  ----  -----------
//! 0x00    4     Signature: "HMX!" (0x21584D48)
//! 0x04    4     Version (0x00000001)
//! 0x08    4     Number of custom robots (n)
//! 0x0C    ...   Robot data blocks:
//!               - Robot index (4 bytes)
//!               - Robot info struct (variable size, ~486 bytes for D2)
//! 0x??    ...   Extra data (custom models, joints, weapons, etc.)
//! ```
//!
//! The extra data section can contain:
//! - Custom polygon models (POF-like data)
//! - Joint positions
//! - Weapon mount points
//! - Animation data
//!
//! # Examples
//!
//! ```no_run
//! use descent_core::hxm::HxmFile;
//!
//! let data = std::fs::read("d2x-xl.hxm")?;
//! let hxm = HxmFile::parse(&data)?;
//!
//! println!("HXM version: {}", hxm.version());
//! println!("Custom robots: {}", hxm.robot_count());
//!
//! // Get custom robot data
//! for (index, robot) in hxm.custom_robots() {
//!     println!("Robot #{}: {} bytes", index, robot.len());
//! }
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

use crate::error::{AssetError, Result};
use crate::io::ReadExt;
use crate::validation::{validate_signature, validate_version};
use std::io::{Cursor, Read};

// HXM file constants
const HXM_SIGNATURE: u32 = 0x21584D48; // "HMX!"
const HXM_VERSION: i32 = 1;

// Robot info structure size (from Descent 2)
// This is the size of the tRobotInfo struct
const ROBOT_INFO_SIZE: usize = 486;

/// HXM file containing custom robot and model data.
///
/// HXM files override or extend HAM file robot definitions with custom data.
/// They are used for expansion packs (like Vertigo) and custom missions.
#[derive(Debug, Clone)]
pub struct HxmFile {
    version: i32,
    /// Custom robot definitions: (robot_index, robot_data)
    custom_robots: Vec<(u32, Vec<u8>)>,
    /// Extra data section containing models, joints, weapons, etc.
    extra_data: Vec<u8>,
}

impl HxmFile {
    /// Parse an HXM file from raw bytes.
    ///
    /// # Arguments
    ///
    /// * `data` - Raw HXM file data
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File signature is invalid
    /// - Version is not 1
    /// - Data is truncated or corrupted
    /// - Robot count is unreasonable (> 1000)
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// let data = std::fs::read("vertigo.hxm")?;
    /// let hxm = HxmFile::parse(&data)?;
    /// println!("Loaded {} custom robots", hxm.robot_count());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(data);
        let file_size = data.len();

        // Parse header
        let signature = cursor.read_u32_le()?;
        validate_signature(signature, HXM_SIGNATURE, "HXM")?;

        let version = cursor.read_i32_le()?;
        validate_version(version, &[HXM_VERSION], "HXM")?;

        // Read number of custom robots
        let robot_count = cursor.read_u32_le()?;
        if robot_count > 1000 {
            return Err(AssetError::InvalidFormat(format!(
                "HXM robot count too large: {} (max 1000)",
                robot_count
            )));
        }

        // Read custom robot data
        let mut custom_robots = Vec::with_capacity(robot_count as usize);
        for _ in 0..robot_count {
            let robot_index = cursor.read_u32_le()?;

            // Read robot info struct
            let mut robot_data = vec![0u8; ROBOT_INFO_SIZE];
            cursor.read_exact(&mut robot_data)?;

            custom_robots.push((robot_index, robot_data));
        }

        // Read remaining extra data
        let current_pos = cursor.position() as usize;
        let remaining = file_size.saturating_sub(current_pos);
        let mut extra_data = vec![0u8; remaining];

        if remaining > 0 {
            cursor.read_exact(&mut extra_data)?;
        }

        Ok(Self {
            version,
            custom_robots,
            extra_data,
        })
    }

    /// Get the HXM file version.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// # let data = vec![0x48, 0x4D, 0x58, 0x21, 1, 0, 0, 0, 0, 0, 0, 0];
    /// let hxm = HxmFile::parse(&data)?;
    /// assert_eq!(hxm.version(), 1);
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn version(&self) -> i32 {
        self.version
    }

    /// Get the number of custom robots defined in this HXM file.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// # let data = vec![];
    /// let hxm = HxmFile::parse(&data)?;
    /// println!("Custom robots: {}", hxm.robot_count());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn robot_count(&self) -> usize {
        self.custom_robots.len()
    }

    /// Get an iterator over custom robot definitions.
    ///
    /// Returns tuples of (robot_index, robot_data).
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// # let data = vec![];
    /// let hxm = HxmFile::parse(&data)?;
    /// for (index, robot_data) in hxm.custom_robots() {
    ///     println!("Robot #{} has {} bytes of data", index, robot_data.len());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn custom_robots(&self) -> impl Iterator<Item = (u32, &[u8])> {
        self.custom_robots
            .iter()
            .map(|(index, data)| (*index, data.as_slice()))
    }

    /// Get robot data for a specific robot index.
    ///
    /// # Arguments
    ///
    /// * `index` - Robot index to look up
    ///
    /// # Returns
    ///
    /// Robot data if found, None otherwise.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// # let data = vec![];
    /// let hxm = HxmFile::parse(&data)?;
    /// if let Some(robot_data) = hxm.get_robot(42) {
    ///     println!("Robot #42 data: {} bytes", robot_data.len());
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn get_robot(&self, index: u32) -> Option<&[u8]> {
        self.custom_robots
            .iter()
            .find(|(idx, _)| *idx == index)
            .map(|(_, data)| data.as_slice())
    }

    /// Get the extra data section.
    ///
    /// The extra data section contains additional custom content such as:
    /// - Polygon models (POF-like data)
    /// - Joint positions
    /// - Weapon mount points
    /// - Animation keyframes
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// # let data = vec![];
    /// let hxm = HxmFile::parse(&data)?;
    /// let extra = hxm.extra_data();
    /// println!("Extra data: {} bytes", extra.len());
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn extra_data(&self) -> &[u8] {
        &self.extra_data
    }

    /// Check if this HXM file has extra data.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use descent_core::hxm::HxmFile;
    /// # let data = vec![];
    /// let hxm = HxmFile::parse(&data)?;
    /// if hxm.has_extra_data() {
    ///     println!("HXM contains custom models/weapons");
    /// }
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn has_extra_data(&self) -> bool {
        !self.extra_data.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_header_only() {
        // Minimal HXM file with no robots
        let data = vec![
            0x48, 0x4D, 0x58, 0x21, // "HMX!" signature
            0x01, 0x00, 0x00, 0x00, // version 1
            0x00, 0x00, 0x00, 0x00, // 0 robots
        ];

        let hxm = HxmFile::parse(&data).unwrap();
        assert_eq!(hxm.version(), 1);
        assert_eq!(hxm.robot_count(), 0);
        assert!(!hxm.has_extra_data());
    }

    #[test]
    fn test_invalid_signature() {
        let data = vec![
            0x00, 0x00, 0x00, 0x00, // Invalid signature
            0x01, 0x00, 0x00, 0x00, // version 1
            0x00, 0x00, 0x00, 0x00, // 0 robots
        ];

        let result = HxmFile::parse(&data);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("signature") || err_msg.contains("HXM"));
    }

    #[test]
    fn test_invalid_version() {
        let data = vec![
            0x48, 0x4D, 0x58, 0x21, // "HMX!" signature
            0x99, 0x00, 0x00, 0x00, // invalid version
            0x00, 0x00, 0x00, 0x00, // 0 robots
        ];

        let result = HxmFile::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }

    #[test]
    fn test_too_many_robots() {
        let data = vec![
            0x48, 0x4D, 0x58, 0x21, // "HMX!" signature
            0x01, 0x00, 0x00, 0x00, // version 1
            0xFF, 0xFF, 0x00, 0x00, // 65535 robots (too many)
        ];

        let result = HxmFile::parse(&data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too large"));
    }

    #[test]
    fn test_get_robot() {
        // HXM with one minimal robot entry
        let mut data = vec![
            0x48, 0x4D, 0x58, 0x21, // "HMX!" signature
            0x01, 0x00, 0x00, 0x00, // version 1
            0x01, 0x00, 0x00, 0x00, // 1 robot
            0x2A, 0x00, 0x00, 0x00, // robot index 42
        ];
        // Add 486 bytes of robot data
        data.extend(vec![0xFF; ROBOT_INFO_SIZE]);

        let hxm = HxmFile::parse(&data).unwrap();
        assert_eq!(hxm.robot_count(), 1);

        let robot = hxm.get_robot(42);
        assert!(robot.is_some());
        assert_eq!(robot.unwrap().len(), ROBOT_INFO_SIZE);

        let missing = hxm.get_robot(99);
        assert!(missing.is_none());
    }

    #[test]
    fn test_extra_data() {
        // HXM with no robots but extra data
        let mut data = vec![
            0x48, 0x4D, 0x58, 0x21, // "HMX!" signature
            0x01, 0x00, 0x00, 0x00, // version 1
            0x00, 0x00, 0x00, 0x00, // 0 robots
        ];
        // Add some extra data
        let extra = b"EXTRA_MODEL_DATA";
        data.extend_from_slice(extra);

        let hxm = HxmFile::parse(&data).unwrap();
        assert!(hxm.has_extra_data());
        assert_eq!(hxm.extra_data().len(), extra.len());
        assert_eq!(hxm.extra_data(), extra);
    }
}
