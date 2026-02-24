//! Common types and utilities shared across all Descent game formats

use crate::error::{AssetError, Result};

/// Descent game version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum GameVersion {
    /// Descent 1 (1995)
    Descent1 = 1,

    /// Descent 2 (1996)
    Descent2 = 2,

    /// Descent 3 (1999)
    Descent3 = 3,
}

impl GameVersion {
    /// Detect game version from file signature or format markers
    pub fn detect_from_hog_signature(signature: &[u8]) -> Result<Self> {
        if signature.len() < 4 {
            return Err(AssetError::InvalidFormat(
                "Signature too short for game detection".to_string(),
            ));
        }

        match &signature[0..4] {
            b"DHF\0" => Ok(Self::Descent1), // D1/D2 use DHF signature
            b"HOG2" => Ok(Self::Descent3),  // D3 uses HOG2 signature
            _ => {
                // No signature might be D1/D2 (they sometimes lack headers)
                // Default to D2 as it's most common
                Ok(Self::Descent2)
            }
        }
    }

    /// Get the game's display name
    pub const fn name(self) -> &'static str {
        match self {
            Self::Descent1 => "Descent 1",
            Self::Descent2 => "Descent 2",
            Self::Descent3 => "Descent 3",
        }
    }

    /// Check if this is a Descent 1 or 2 game (legacy format)
    pub const fn is_d1_d2(self) -> bool {
        matches!(self, Self::Descent1 | Self::Descent2)
    }

    /// Check if this is Descent 3 (Outrage engine)
    pub const fn is_d3(self) -> bool {
        matches!(self, Self::Descent3)
    }
}

impl std::fmt::Display for GameVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

impl From<GameVersion> for u8 {
    fn from(version: GameVersion) -> Self {
        version as u8
    }
}

impl TryFrom<u8> for GameVersion {
    type Error = AssetError;

    fn try_from(value: u8) -> Result<Self> {
        match value {
            1 => Ok(Self::Descent1),
            2 => Ok(Self::Descent2),
            3 => Ok(Self::Descent3),
            _ => Err(AssetError::InvalidFormat(format!(
                "Invalid game version: {}",
                value
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_version_detection() {
        assert_eq!(
            GameVersion::detect_from_hog_signature(b"DHF\0").unwrap(),
            GameVersion::Descent1
        );
        assert_eq!(
            GameVersion::detect_from_hog_signature(b"HOG2").unwrap(),
            GameVersion::Descent3
        );
    }

    #[test]
    fn test_game_version_names() {
        assert_eq!(GameVersion::Descent1.name(), "Descent 1");
        assert_eq!(GameVersion::Descent2.name(), "Descent 2");
        assert_eq!(GameVersion::Descent3.name(), "Descent 3");
    }

    #[test]
    fn test_game_version_checks() {
        assert!(GameVersion::Descent1.is_d1_d2());
        assert!(GameVersion::Descent2.is_d1_d2());
        assert!(!GameVersion::Descent3.is_d1_d2());
        assert!(GameVersion::Descent3.is_d3());
    }

    #[test]
    fn test_game_version_conversion() {
        assert_eq!(u8::from(GameVersion::Descent1), 1);
        assert_eq!(u8::from(GameVersion::Descent2), 2);
        assert_eq!(u8::from(GameVersion::Descent3), 3);

        assert_eq!(GameVersion::try_from(1).unwrap(), GameVersion::Descent1);
        assert_eq!(GameVersion::try_from(2).unwrap(), GameVersion::Descent2);
        assert_eq!(GameVersion::try_from(3).unwrap(), GameVersion::Descent3);
        assert!(GameVersion::try_from(4).is_err());
    }
}
