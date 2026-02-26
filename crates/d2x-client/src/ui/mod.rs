//! Descent-style menu UI system
//!
//! Based on D2X-XL's menu implementation, this module provides a retro text-based
//! menu system with:
//! - Various menu item types (text, button, checkbox, slider, input, etc.)
//! - Keyboard navigation
//! - Mouse support
//! - Descent's classic visual style

pub mod menu;
pub mod menu_item;

pub use menu::Menu;
pub use menu::MenuPlugin;
pub use menu::MenuState;
pub use menu_item::{MenuItem, MenuItemType};
