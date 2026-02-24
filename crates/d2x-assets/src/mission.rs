//! Mission file parser for Descent 1 (.msn) and Descent 2 (.mn2) missions.
//!
//! Mission files are text-based configuration files that define:
//! - Mission metadata (name, type)
//! - Level list (regular and secret levels)
//! - Associated HOG file
//! - Briefing and ending text files
//!
//! The format is identical between .msn (D1) and .mn2 (D2), only the file extension differs.
//!
//! # Example
//!
//! ```ignore
//! use d2x_assets::MissionFile;
//!
//! let data = std::fs::read_to_string("mymission.mn2")?;
//! let mission = MissionFile::parse(&data)?;
//!
//! println!("Mission: {}", mission.name);
//! println!("Levels: {}", mission.levels.len());
//! for (i, level) in mission.levels.iter().enumerate() {
//!     println!("  Level {}: {}", i + 1, level);
//! }
//! ```

use crate::error::{AssetError, Result};

/// Mission file parser for Descent 1 (.msn) and Descent 2 (.mn2).
///
/// Mission files are text-based configuration files using an INI-like format
/// with key-value pairs and comments starting with `;`.
#[derive(Debug, Clone, PartialEq)]
pub struct MissionFile {
    /// Mission name (from `name` field, required)
    pub name: String,

    /// Mission type (e.g., "normal", "anarchy")
    pub mission_type: Option<String>,

    /// Associated HOG archive filename
    pub hog_file: Option<String>,

    /// Briefing text filename (max 12 characters)
    pub briefing_file: Option<String>,

    /// Ending text filename (max 12 characters)
    pub ending_file: Option<String>,

    /// Regular level filenames (e.g., "level01.rdl", "level02.rl2")
    pub levels: Vec<String>,

    /// Secret level data (level filename and base levels it links from)
    pub secret_levels: Vec<SecretLevel>,

    /// Enhancement level (0 = standard, 1 = xname, 2 = zname, 3 = d2x-name)
    pub enhancement_level: u8,
}

/// Secret level information.
///
/// Secret levels are linked from one or more regular levels.
/// For example, a secret level might be accessible from level 10 and level 21.
#[derive(Debug, Clone, PartialEq)]
pub struct SecretLevel {
    /// Secret level filename (e.g., "levels1.rdl")
    pub filename: String,

    /// Base level numbers this secret level can be accessed from (e.g., [10, 21])
    /// These are 1-indexed level numbers
    pub linked_from: Vec<u32>,
}

impl MissionFile {
    /// Parse a mission file from text content.
    ///
    /// # Arguments
    ///
    /// * `content` - Mission file content as a string
    ///
    /// # Returns
    ///
    /// Parsed mission file or error if invalid format.
    ///
    /// # Example
    ///
    /// ```
    /// # use d2x_assets::MissionFile;
    /// let content = "name = Test Mission\ntype = normal\nnum_levels = 1\nlevel01.rdl\n";
    /// let mission = MissionFile::parse(content).unwrap();
    /// assert_eq!(mission.name, "Test Mission");
    /// assert_eq!(mission.levels.len(), 1);
    /// ```
    pub fn parse(content: &str) -> Result<Self> {
        let mut mission = MissionFile {
            name: String::new(),
            mission_type: None,
            hog_file: None,
            briefing_file: None,
            ending_file: None,
            levels: Vec::new(),
            secret_levels: Vec::new(),
            enhancement_level: 0,
        };

        let mut lines = content.lines().peekable();

        while let Some(line) = lines.next() {
            // Remove comments (everything after ';')
            let line = if let Some(pos) = line.find(';') {
                &line[..pos]
            } else {
                line
            };

            let line = line.trim();

            // Skip empty lines
            if line.is_empty() {
                continue;
            }

            // Parse key-value pairs
            if let Some((key, value)) = Self::split_key_value(line) {
                match key.to_lowercase().as_str() {
                    "name" => {
                        mission.name = value.to_string();
                    }
                    "xname" => {
                        mission.name = value.to_string();
                        mission.enhancement_level = 1;
                    }
                    "zname" => {
                        mission.name = value.to_string();
                        mission.enhancement_level = 2;
                    }
                    "d2x-name" => {
                        mission.name = value.to_string();
                        mission.enhancement_level = 3;
                    }
                    "type" => {
                        mission.mission_type = Some(value.to_string());
                    }
                    "hog" => {
                        mission.hog_file = Some(value.to_string());
                    }
                    "briefing" => {
                        if value.len() <= 12 {
                            mission.briefing_file = Some(value.to_string());
                        } else {
                            return Err(AssetError::InvalidFormat(format!(
                                "Briefing filename too long (max 12 chars): {}",
                                value
                            )));
                        }
                    }
                    "ending" => {
                        if value.len() <= 12 {
                            mission.ending_file = Some(value.to_string());
                        } else {
                            return Err(AssetError::InvalidFormat(format!(
                                "Ending filename too long (max 12 chars): {}",
                                value
                            )));
                        }
                    }
                    "num_levels" => {
                        let count: usize = value.parse().map_err(|_| {
                            AssetError::InvalidFormat(format!("Invalid num_levels: {}", value))
                        })?;

                        if count > 100 {
                            return Err(AssetError::InvalidFormat(format!(
                                "Too many levels (max 100): {}",
                                count
                            )));
                        }

                        // Read level filenames (one per line)
                        for _ in 0..count {
                            if let Some(level_line) = lines.next() {
                                // Remove comments and trim
                                let level_line = if let Some(pos) = level_line.find(';') {
                                    &level_line[..pos]
                                } else {
                                    level_line
                                };
                                let level_name = level_line.trim();

                                if !level_name.is_empty() {
                                    if level_name.len() > 255 {
                                        return Err(AssetError::InvalidFormat(format!(
                                            "Level filename too long: {}",
                                            level_name
                                        )));
                                    }
                                    mission.levels.push(level_name.to_string());
                                } else {
                                    return Err(AssetError::InvalidFormat(
                                        "Empty level filename".to_string(),
                                    ));
                                }
                            } else {
                                return Err(AssetError::InvalidFormat(format!(
                                    "Expected {} levels, got {}",
                                    count,
                                    mission.levels.len()
                                )));
                            }
                        }
                    }
                    "num_secrets" => {
                        let count: usize = value.parse().map_err(|_| {
                            AssetError::InvalidFormat(format!("Invalid num_secrets: {}", value))
                        })?;

                        if count > 20 {
                            return Err(AssetError::InvalidFormat(format!(
                                "Too many secret levels (max 20): {}",
                                count
                            )));
                        }

                        // Read secret level data (one per line)
                        for _ in 0..count {
                            if let Some(secret_line) = lines.next() {
                                // Remove comments and trim
                                let secret_line = if let Some(pos) = secret_line.find(';') {
                                    &secret_line[..pos]
                                } else {
                                    secret_line
                                };
                                let secret_line = secret_line.trim();

                                if !secret_line.is_empty() {
                                    let secret = Self::parse_secret_level(secret_line)?;
                                    mission.secret_levels.push(secret);
                                } else {
                                    return Err(AssetError::InvalidFormat(
                                        "Empty secret level line".to_string(),
                                    ));
                                }
                            } else {
                                return Err(AssetError::InvalidFormat(format!(
                                    "Expected {} secret levels, got {}",
                                    count,
                                    mission.secret_levels.len()
                                )));
                            }
                        }
                    }
                    _ => {
                        // Unknown key, ignore (for forward compatibility)
                    }
                }
            }
        }

        // Validate required fields
        if mission.name.is_empty() {
            return Err(AssetError::InvalidFormat(
                "Missing required field: name".to_string(),
            ));
        }

        Ok(mission)
    }

    /// Split a line into key and value parts.
    ///
    /// Format: `key = value`
    fn split_key_value(line: &str) -> Option<(&str, &str)> {
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let value = line[pos + 1..].trim();
            if !key.is_empty() && !value.is_empty() {
                return Some((key, value));
            }
        }
        None
    }

    /// Parse a secret level line.
    ///
    /// Format: `levelname.rdl,1,10,21`
    /// The first part is the filename, subsequent parts are base level numbers.
    fn parse_secret_level(line: &str) -> Result<SecretLevel> {
        let parts: Vec<&str> = line.split(',').collect();

        if parts.is_empty() {
            return Err(AssetError::InvalidFormat(
                "Empty secret level line".to_string(),
            ));
        }

        let filename = parts[0].trim();

        if filename.is_empty() {
            return Err(AssetError::InvalidFormat(
                "Empty secret level filename".to_string(),
            ));
        }

        if filename.len() > 255 {
            return Err(AssetError::InvalidFormat(format!(
                "Secret level filename too long: {}",
                filename
            )));
        }

        let mut linked_from = Vec::new();

        // Parse base level numbers
        for part in &parts[1..] {
            let level_num: u32 = part
                .trim()
                .parse()
                .map_err(|_| AssetError::InvalidFormat(format!("Invalid level number: {}", part)))?;

            if level_num < 1 {
                return Err(AssetError::InvalidFormat(format!(
                    "Invalid level number (must be >= 1): {}",
                    level_num
                )));
            }

            linked_from.push(level_num);
        }

        if linked_from.is_empty() {
            return Err(AssetError::InvalidFormat(
                "Secret level must link from at least one base level".to_string(),
            ));
        }

        Ok(SecretLevel {
            filename: filename.to_string(),
            linked_from,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_mission() {
        let content = "name = Test Mission\n\
                      type = normal\n\
                      num_levels = 3\n\
                      level01.rdl\n\
                      level02.rdl\n\
                      level03.rdl\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.name, "Test Mission");
        assert_eq!(mission.mission_type, Some("normal".to_string()));
        assert_eq!(mission.levels.len(), 3);
        assert_eq!(mission.levels[0], "level01.rdl");
        assert_eq!(mission.levels[1], "level02.rdl");
        assert_eq!(mission.levels[2], "level03.rdl");
    }

    #[test]
    fn test_parse_with_comments() {
        let content = "; This is a comment\n\
                      name = Test Mission ; inline comment\n\
                      num_levels = 1\n\
                      level01.rdl ; level comment\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.name, "Test Mission");
        assert_eq!(mission.levels.len(), 1);
        assert_eq!(mission.levels[0], "level01.rdl");
    }

    #[test]
    fn test_parse_with_hog() {
        let content = "name = Test Mission\n\
                      hog = mymission.hog\n\
                      num_levels = 1\n\
                      level01.rdl\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.hog_file, Some("mymission.hog".to_string()));
    }

    #[test]
    fn test_parse_with_briefing() {
        let content = "name = Test Mission\n\
                      briefing = brief.txb\n\
                      ending = ending.txb\n\
                      num_levels = 1\n\
                      level01.rdl\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.briefing_file, Some("brief.txb".to_string()));
        assert_eq!(mission.ending_file, Some("ending.txb".to_string()));
    }

    #[test]
    fn test_parse_with_secret_levels() {
        let content = "name = Test Mission\n\
                      num_levels = 3\n\
                      level01.rdl\n\
                      level02.rdl\n\
                      level03.rdl\n\
                      num_secrets = 2\n\
                      levels1.rdl,1\n\
                      levels2.rdl,2,3\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.secret_levels.len(), 2);
        assert_eq!(mission.secret_levels[0].filename, "levels1.rdl");
        assert_eq!(mission.secret_levels[0].linked_from, vec![1]);
        assert_eq!(mission.secret_levels[1].filename, "levels2.rdl");
        assert_eq!(mission.secret_levels[1].linked_from, vec![2, 3]);
    }

    #[test]
    fn test_parse_enhanced_mission() {
        let content = "xname = Enhanced Mission\n\
                      num_levels = 1\n\
                      level01.rdl\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.name, "Enhanced Mission");
        assert_eq!(mission.enhancement_level, 1);
    }

    #[test]
    fn test_parse_d2x_mission() {
        let content = "d2x-name = D2X-XL Mission\n\
                      num_levels = 1\n\
                      level01.rdl\n";

        let mission = MissionFile::parse(content).unwrap();
        assert_eq!(mission.name, "D2X-XL Mission");
        assert_eq!(mission.enhancement_level, 3);
    }

    #[test]
    fn test_parse_missing_name() {
        let content = "type = normal\n\
                      num_levels = 1\n\
                      level01.rdl\n";

        let result = MissionFile::parse(content);
        assert!(result.is_err());
        assert!(matches!(result, Err(AssetError::InvalidFormat(_))));
    }

    #[test]
    fn test_parse_briefing_too_long() {
        let content = "name = Test\n\
                      briefing = verylongfilename.txt\n\
                      num_levels = 1\n\
                      level01.rdl\n";

        let result = MissionFile::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_secret_without_links() {
        let content = "name = Test\n\
                      num_levels = 1\n\
                      level01.rdl\n\
                      num_secrets = 1\n\
                      levels1.rdl\n";

        let result = MissionFile::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_empty_content() {
        let content = "";
        let result = MissionFile::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_invalid_num_levels() {
        let content = "name = Test\n\
                      num_levels = not_a_number\n";

        let result = MissionFile::parse(content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_too_many_levels() {
        let content = "name = Test\n\
                      num_levels = 101\n";

        let result = MissionFile::parse(content);
        assert!(result.is_err());
    }
}
