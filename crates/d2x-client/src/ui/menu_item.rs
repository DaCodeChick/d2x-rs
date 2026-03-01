//! Menu item types and rendering
//!
//! Replicates D2X-XL's CMenuItem class

use bevy::prelude::*;

/// Menu item type (from D2X-XL NM_TYPE_* defines)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(dead_code)] // Future menu features
pub enum MenuItemType {
    /// Menu item - when Enter is hit, menu returns this item number
    Menu,
    /// Input box - fills text field, requires text_len
    Input,
    /// Checkbox - get/set status via value (1=on, 0=off)
    Check,
    /// Radio button - only 1 in group can be set at a time
    Radio,
    /// Line of text that does nothing
    Text,
    /// Numeric entry counter - changes from min_value to max_value
    Number,
    /// Input box that you hit Enter to edit
    InputMenu,
    /// Slider from min_value to max_value
    Slider,
    /// Gauge/progress bar from min_value to max_value
    Gauge,
}

/// Menu item component (mirrors D2X-XL's CMenuItem)
#[derive(Component, Clone)]
pub struct MenuItem {
    /// Item type
    pub item_type: MenuItemType,
    /// Radio button group
    pub group: i32,
    /// Text to display
    pub text: String,
    /// Value (for checkboxes, radio buttons, sliders)
    pub value: i32,
    /// Minimum value (for sliders, number inputs)
    pub min_value: i32,
    /// Maximum value (for sliders, number inputs)
    pub max_value: i32,
    /// Maximum text length for input boxes
    #[allow(dead_code)] // Future input box feature
    pub text_len: usize,
    /// Whether item is unavailable/grayed out
    pub unavailable: bool,
    /// Whether item is centered
    #[allow(dead_code)] // Future layout feature
    pub centered: bool,
    /// Hotkey character (from D2X-XL's m_nKey)
    #[allow(dead_code)] // Future hotkey feature
    pub hotkey: Option<char>,
    /// Item ID for lookups
    #[allow(dead_code)] // Future item lookup feature
    pub id: String,
}

impl MenuItem {
    /// Create a text label item
    pub fn text(text: impl Into<String>) -> Self {
        Self {
            item_type: MenuItemType::Text,
            text: text.into(),
            ..default()
        }
    }

    /// Create a menu button item
    pub fn menu(text: impl Into<String>) -> Self {
        Self {
            item_type: MenuItemType::Menu,
            text: text.into(),
            ..default()
        }
    }

    /// Create a checkbox item
    pub fn checkbox(text: impl Into<String>, checked: bool) -> Self {
        Self {
            item_type: MenuItemType::Check,
            text: text.into(),
            value: if checked { 1 } else { 0 },
            ..default()
        }
    }

    /// Create a slider item
    pub fn slider(text: impl Into<String>, value: i32, min: i32, max: i32) -> Self {
        Self {
            item_type: MenuItemType::Slider,
            text: text.into(),
            value,
            min_value: min,
            max_value: max,
            ..default()
        }
    }

    /// Create an input box
    #[allow(dead_code)] // Future input box feature
    pub fn input(text: impl Into<String>, max_len: usize) -> Self {
        Self {
            item_type: MenuItemType::Input,
            text: text.into(),
            text_len: max_len,
            ..default()
        }
    }

    /// Set hotkey for this item
    #[allow(dead_code)] // Future hotkey feature
    pub fn with_hotkey(mut self, key: char) -> Self {
        self.hotkey = Some(key);
        self
    }

    /// Set item ID for lookups
    #[allow(dead_code)] // Future item lookup feature
    pub fn with_id(mut self, id: impl Into<String>) -> Self {
        self.id = id.into();
        self
    }

    /// Set centered flag
    #[allow(dead_code)] // Future layout feature
    pub fn centered(mut self) -> Self {
        self.centered = true;
        self
    }

    /// Check if item is selectable
    pub fn is_selectable(&self) -> bool {
        !matches!(self.item_type, MenuItemType::Text) && !self.unavailable && !self.text.is_empty()
    }

    /// Get display text with special characters (from D2X-XL)
    pub fn display_text(&self) -> String {
        match self.item_type {
            MenuItemType::Check => {
                let checkbox = if self.value != 0 { "☑" } else { "☐" };
                format!("{} {}", checkbox, self.text)
            }
            MenuItemType::Radio => {
                let radio = if self.value != 0 { "●" } else { "○" };
                format!("{} {}", radio, self.text)
            }
            MenuItemType::Slider => {
                let range = (self.max_value - self.min_value) as f32;
                let percent = if range > 0.0 {
                    (self.value - self.min_value) as f32 / range
                } else {
                    0.0
                };
                let slider_width = 10;
                let filled = (percent * slider_width as f32) as usize;
                let slider = format!(
                    "[{}{}]",
                    "■".repeat(filled),
                    "□".repeat(slider_width - filled)
                );
                format!("{}: {}", self.text, slider)
            }
            MenuItemType::Gauge => {
                let range = (self.max_value - self.min_value) as f32;
                let percent = if range > 0.0 {
                    (self.value - self.min_value) as f32 / range
                } else {
                    0.0
                };
                format!("{}: {}%", self.text, (percent * 100.0) as i32)
            }
            _ => self.text.clone(),
        }
    }
}

impl Default for MenuItem {
    fn default() -> Self {
        Self {
            item_type: MenuItemType::Text,
            group: 0,
            text: String::new(),
            value: 0,
            min_value: 0,
            max_value: 0,
            text_len: 0,
            unavailable: false,
            centered: false,
            hotkey: None,
            id: String::new(),
        }
    }
}
