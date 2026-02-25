//! Player profile parsing for Descent 1/2 and D2X-XL.
//!
//! Player profiles store player identity, game progress, control configuration,
//! and other settings. This module supports two formats:
//!
//! 1. **Binary .PLR format** (Descent 1/2) - Binary structures with player data
//! 2. **Text .PLX format** (D2X-XL) - Text-based key=value configuration
//!
//! # Binary Format (Descent 1/2)
//!
//! The original Descent 1/2 player files (.PLR) use a binary format:
//! - File signature: "PLYR" (4 bytes)
//! - File version number (varies by game)
//! - Player data structures
//! - Control configuration
//! - Mission progress data
//!
//! # Text Format (D2X-XL)
//!
//! D2X-XL replaced binary .PLR files with text .PLX files:
//! - Simple key=value pairs
//! - One parameter per line
//! - Human-readable and editable
//! - Contains game options, controls, and player settings
//!
//! # Examples
//!
//! ```no_run
//! use descent_core::player::{PlayerProfile, PlxProfile};
//! use std::fs;
//!
//! // Parse D2X-XL text profile
//! let data = fs::read_to_string("player.plx").unwrap();
//! let profile = PlxProfile::parse(&data).unwrap();
//! println!("Profile has {} settings", profile.params().len());
//! ```

use crate::error::{AssetError, Result};
use std::collections::HashMap;

// Constants from D2X-XL source (playerprofile.h)
/// Compatible with original Descent 2 file version
pub const COMPATIBLE_PLAYER_FILE_VERSION: u32 = 17;
/// D2 Windows 95 version
pub const D2W95_PLAYER_FILE_VERSION: u32 = 24;
/// D2X-W32 version
pub const D2XW32_PLAYER_FILE_VERSION: u32 = 45;
/// D2X-XL version
pub const D2XXL_PLAYER_FILE_VERSION: u32 = 161;

/// Maximum callsign length (player name)
pub const CALLSIGN_LEN: usize = 8;
/// Number of save game slots
pub const N_SAVE_SLOTS: usize = 10;
/// Maximum mission name length
pub const GAME_NAME_LEN: usize = 25;

/// Player profile data (abstract interface)
///
/// This trait provides a common interface for accessing player profile data
/// regardless of the underlying format (binary .PLR or text .PLX).
pub trait PlayerProfile {
    /// Get the player's callsign (name)
    fn callsign(&self) -> &str;

    /// Get profile format version
    fn version(&self) -> u32;
}

/// D2X-XL text-based player profile (.PLX format)
///
/// D2X-XL uses a text-based configuration file format where each setting
/// is stored as a key=value pair on a separate line. This format is
/// human-readable and editable.
///
/// # Format
///
/// ```text
/// gameData.renderData.screen.m_w=640
/// gameData.renderData.screen.m_h=480
/// gameStates.render.bShowFrameRate=0
/// gameOptions[0].render.nQuality=2
/// keyboard.Fire primary[0].value=-1
/// ```
///
/// # Examples
///
/// ```no_run
/// use descent_core::player::PlxProfile;
///
/// let data = "gameData.renderData.screen.m_w=1920\n\
///             gameData.renderData.screen.m_h=1080\n\
///             gameStates.render.bShowFrameRate=1\n";
/// let profile = PlxProfile::parse(data).unwrap();
///
/// assert_eq!(profile.get("gameData.renderData.screen.m_w"), Some(&"1920".to_string()));
/// assert_eq!(profile.params().len(), 3);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PlxProfile {
    /// Parameter map: key -> value
    params: HashMap<String, String>,
    /// Format version (if specified in file)
    version: Option<u32>,
}

impl PlxProfile {
    /// Create a new empty PLX profile
    pub fn new() -> Self {
        Self {
            params: HashMap::new(),
            version: None,
        }
    }

    /// Parse a D2X-XL text profile from a string
    ///
    /// # Format
    ///
    /// Each line contains a key=value pair:
    /// ```text
    /// parameter.name=value
    /// ```
    ///
    /// Empty lines and lines without '=' are ignored.
    ///
    /// # Errors
    ///
    /// Returns an error if the data cannot be parsed.
    pub fn parse(data: &str) -> Result<Self> {
        let mut profile = Self::new();

        profile.params = data
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                let eq_pos = line.find('=')?;
                let key = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();

                if key.is_empty() {
                    None
                } else {
                    Some((key.to_string(), value.to_string()))
                }
            })
            .collect();

        Ok(profile)
    }

    /// Get a parameter value by key
    ///
    /// Returns `None` if the key doesn't exist.
    pub fn get(&self, key: &str) -> Option<&String> {
        self.params.get(key)
    }

    /// Get a parameter value as an integer
    ///
    /// Returns `None` if the key doesn't exist or the value cannot be parsed.
    pub fn get_int(&self, key: &str) -> Option<i32> {
        self.params.get(key)?.parse().ok()
    }

    /// Get a parameter value as a boolean (0 = false, non-zero = true)
    ///
    /// Returns `None` if the key doesn't exist or the value cannot be parsed.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.get_int(key).map(|v| v != 0)
    }

    /// Set a parameter value
    pub fn set(&mut self, key: String, value: String) {
        self.params.insert(key, value);
    }

    /// Get all parameters
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Get the number of parameters
    pub fn len(&self) -> usize {
        self.params.len()
    }

    /// Check if the profile is empty
    pub fn is_empty(&self) -> bool {
        self.params.is_empty()
    }

    /// Serialize to PLX format
    ///
    /// Returns a string representation suitable for writing to a .plx file.
    pub fn serialize(&self) -> String {
        let mut lines: Vec<String> = self
            .params
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect();

        // Sort for consistent output
        lines.sort();

        lines.join("\n") + "\n"
    }
}

impl Default for PlxProfile {
    fn default() -> Self {
        Self::new()
    }
}

impl PlayerProfile for PlxProfile {
    fn callsign(&self) -> &str {
        // PLX files are named after the callsign (filename.plx)
        // The callsign is not stored in the file itself
        ""
    }

    fn version(&self) -> u32 {
        self.version.unwrap_or(D2XXL_PLAYER_FILE_VERSION)
    }
}

/// Binary player profile (.PLR format) - Descent 1/2
///
/// The original Descent 1 and 2 games used a binary .PLR format to store
/// player profiles. This format contains:
/// - Player callsign (name)
/// - Mission progress (highest levels reached)
/// - Control configuration
/// - Statistics and preferences
///
/// **Note**: Full binary .PLR parsing is not yet implemented. This is a
/// placeholder for future implementation. The binary format requires
/// detailed reverse engineering or access to original source code.
///
/// # Binary Structure (Preliminary)
///
/// ```text
/// Offset  Size  Description
/// ------  ----  -----------
/// 0x00    4     File signature "PLYR"
/// 0x04    4     File version number
/// 0x08    ?     Player data (version-dependent)
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct PlrProfile {
    /// Player callsign (8 characters max)
    callsign: String,
    /// File version
    version: u32,
    /// Raw data (not yet fully parsed)
    data: Vec<u8>,
}

impl PlrProfile {
    /// Create a new empty PLR profile
    pub fn new(callsign: String, version: u32) -> Self {
        Self {
            callsign,
            version,
            data: Vec::new(),
        }
    }

    /// Parse a binary .PLR file
    ///
    /// **Note**: This is a placeholder implementation. Full binary parsing
    /// is not yet supported.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be parsed.
    pub fn parse(_data: &[u8]) -> Result<Self> {
        // TODO: Implement binary PLR parsing
        // This requires detailed format specification from original source
        Err(AssetError::UnsupportedFormat(
            "Binary .PLR format parsing not yet implemented. Use D2X-XL .PLX format instead."
                .to_string(),
        ))
    }

    /// Get the player's callsign
    pub fn get_callsign(&self) -> &str {
        &self.callsign
    }

    /// Get the file version
    pub fn get_version(&self) -> u32 {
        self.version
    }

    /// Get the raw data
    pub fn data(&self) -> &[u8] {
        &self.data
    }
}

impl PlayerProfile for PlrProfile {
    fn callsign(&self) -> &str {
        &self.callsign
    }

    fn version(&self) -> u32 {
        self.version
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plx_parse_empty() {
        let profile = PlxProfile::parse("").unwrap();
        assert_eq!(profile.len(), 0);
        assert!(profile.is_empty());
    }

    #[test]
    fn test_plx_parse_single_param() {
        let data = "gameData.renderData.screen.m_w=1920\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.len(), 1);
        assert_eq!(
            profile.get("gameData.renderData.screen.m_w"),
            Some(&"1920".to_string())
        );
    }

    #[test]
    fn test_plx_parse_multiple_params() {
        let data = "\
            gameData.renderData.screen.m_w=1920\n\
            gameData.renderData.screen.m_h=1080\n\
            gameStates.render.bShowFrameRate=1\n\
        ";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.len(), 3);
        assert_eq!(
            profile.get("gameData.renderData.screen.m_w"),
            Some(&"1920".to_string())
        );
        assert_eq!(
            profile.get("gameData.renderData.screen.m_h"),
            Some(&"1080".to_string())
        );
        assert_eq!(
            profile.get("gameStates.render.bShowFrameRate"),
            Some(&"1".to_string())
        );
    }

    #[test]
    fn test_plx_parse_with_spaces() {
        let data = "  key1  =  value1  \n key2=value2\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.len(), 2);
        assert_eq!(profile.get("key1"), Some(&"value1".to_string()));
        assert_eq!(profile.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_plx_parse_empty_lines() {
        let data = "\n\n  key1=value1  \n\n\n  key2=value2  \n\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.len(), 2);
    }

    #[test]
    fn test_plx_parse_no_equals() {
        let data = "not_a_valid_line\nkey1=value1\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.len(), 1);
        assert_eq!(profile.get("key1"), Some(&"value1".to_string()));
    }

    #[test]
    fn test_plx_get_int() {
        let data = "width=1920\nheight=1080\ninvalid=not_a_number\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.get_int("width"), Some(1920));
        assert_eq!(profile.get_int("height"), Some(1080));
        assert_eq!(profile.get_int("invalid"), None);
        assert_eq!(profile.get_int("missing"), None);
    }

    #[test]
    fn test_plx_get_bool() {
        let data = "enabled=1\ndisabled=0\ninvalid=not_a_bool\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.get_bool("enabled"), Some(true));
        assert_eq!(profile.get_bool("disabled"), Some(false));
        assert_eq!(profile.get_bool("invalid"), None);
        assert_eq!(profile.get_bool("missing"), None);
    }

    #[test]
    fn test_plx_set() {
        let mut profile = PlxProfile::new();
        profile.set("key1".to_string(), "value1".to_string());
        profile.set("key2".to_string(), "value2".to_string());
        assert_eq!(profile.len(), 2);
        assert_eq!(profile.get("key1"), Some(&"value1".to_string()));
        assert_eq!(profile.get("key2"), Some(&"value2".to_string()));
    }

    #[test]
    fn test_plx_serialize() {
        let mut profile = PlxProfile::new();
        profile.set("key2".to_string(), "value2".to_string());
        profile.set("key1".to_string(), "value1".to_string());

        let serialized = profile.serialize();

        // Should be sorted
        assert!(serialized.contains("key1=value1"));
        assert!(serialized.contains("key2=value2"));
        assert!(serialized.starts_with("key1=") || serialized.starts_with("key2="));
    }

    #[test]
    fn test_plx_roundtrip() {
        let original = "\
            gameData.renderData.screen.m_w=1920\n\
            gameData.renderData.screen.m_h=1080\n\
            gameStates.render.bShowFrameRate=1\n\
        ";

        let profile = PlxProfile::parse(original).unwrap();
        let serialized = profile.serialize();
        let reparsed = PlxProfile::parse(&serialized).unwrap();

        assert_eq!(profile.len(), reparsed.len());
        assert_eq!(profile.params(), reparsed.params());
    }

    #[test]
    fn test_plx_array_notation() {
        let data = "\
            extraGameInfo[0].bAutoBalanceTeams=0\n\
            extraGameInfo[1].bAutoBalanceTeams=1\n\
            gameOptions[0].render.nQuality=2\n\
        ";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(profile.len(), 3);
        assert_eq!(
            profile.get("extraGameInfo[0].bAutoBalanceTeams"),
            Some(&"0".to_string())
        );
        assert_eq!(
            profile.get("extraGameInfo[1].bAutoBalanceTeams"),
            Some(&"1".to_string())
        );
        assert_eq!(
            profile.get("gameOptions[0].render.nQuality"),
            Some(&"2".to_string())
        );
    }

    #[test]
    fn test_plx_negative_values() {
        let data = "keyboard.Pitch forward[0].value=-56\n";
        let profile = PlxProfile::parse(data).unwrap();
        assert_eq!(
            profile.get_int("keyboard.Pitch forward[0].value"),
            Some(-56)
        );
    }

    #[test]
    fn test_plr_not_implemented() {
        let data = b"PLYR\x11\x00\x00\x00";
        let result = PlrProfile::parse(data);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            AssetError::UnsupportedFormat(_)
        ));
    }

    #[test]
    fn test_plr_new() {
        let profile = PlrProfile::new("PLAYER".to_string(), COMPATIBLE_PLAYER_FILE_VERSION);
        assert_eq!(profile.callsign(), "PLAYER");
        assert_eq!(profile.version(), COMPATIBLE_PLAYER_FILE_VERSION);
    }

    #[test]
    fn test_player_profile_trait() {
        let plx = PlxProfile::new();
        assert_eq!(plx.version(), D2XXL_PLAYER_FILE_VERSION);

        let plr = PlrProfile::new("TEST".to_string(), COMPATIBLE_PLAYER_FILE_VERSION);
        assert_eq!(plr.callsign(), "TEST");
        assert_eq!(plr.version(), COMPATIBLE_PLAYER_FILE_VERSION);
    }
}
