//! Menu system (mirrors D2X-XL's CMenu class)

use super::{MenuItem, MenuItemType};
use bevy::prelude::*;

/// Menu state resource
#[derive(Resource, Default)]
pub struct MenuState {
    /// Currently active menu
    pub active_menu: Option<Entity>,
    /// Currently selected item index
    pub selected_index: usize,
    /// Menu is visible
    pub visible: bool,
    /// Slider being dragged (menu item index)
    pub dragging_slider: Option<usize>,
}

/// Menu component (mirrors D2X-XL's CMenu class)
#[derive(Component)]
pub struct Menu {
    /// Menu title
    pub title: String,
    /// Menu subtitle
    pub subtitle: Option<String>,
    /// Menu items
    pub items: Vec<MenuItem>,
    /// Top visible item for scrolling
    pub top_choice: usize,
    /// Background color (Descent blue)
    pub bg_color: Color,
}

impl Menu {
    /// Create a new menu
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            subtitle: None,
            items: Vec::new(),
            top_choice: 0,
            bg_color: Color::srgb(0.0, 0.0, 0.2), // Dark blue like Descent
        }
    }

    /// Set subtitle
    pub fn with_subtitle(mut self, subtitle: impl Into<String>) -> Self {
        self.subtitle = Some(subtitle.into());
        self
    }

    /// Add menu item
    pub fn add_item(mut self, item: MenuItem) -> Self {
        self.items.push(item);
        self
    }

    /// Add multiple items
    pub fn add_items(mut self, items: Vec<MenuItem>) -> Self {
        self.items.extend(items);
        self
    }

    /// Get number of selectable items
    pub fn selectable_count(&self) -> usize {
        self.items
            .iter()
            .filter(|item| item.is_selectable())
            .count()
    }

    /// Find item by ID
    pub fn find_item(&self, id: &str) -> Option<&MenuItem> {
        self.items.iter().find(|item| item.id == id)
    }

    /// Find item by ID (mutable)
    pub fn find_item_mut(&mut self, id: &str) -> Option<&mut MenuItem> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    /// Get value of item by ID
    pub fn get_value(&self, id: &str) -> Option<i32> {
        self.find_item(id).map(|item| item.value)
    }

    /// Set value of item by ID
    pub fn set_value(&mut self, id: &str, value: i32) {
        if let Some(item) = self.find_item_mut(id) {
            item.value = value;
        }
    }
}

/// Menu plugin
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuState>().add_systems(
            Update,
            (
                handle_menu_input,
                handle_menu_hover,
                handle_menu_click,
                handle_slider_drag,
                render_menu,
            )
                .chain(),
        );
    }
}

/// Handle menu input
fn handle_menu_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut menu_state: ResMut<MenuState>,
    mut menu_query: Query<&mut Menu>,
) {
    if !menu_state.visible || menu_state.active_menu.is_none() {
        return;
    }

    let menu_entity = menu_state.active_menu.unwrap();
    let Ok(mut menu) = menu_query.get_mut(menu_entity) else {
        return;
    };

    // Navigate up
    if keyboard.just_pressed(KeyCode::ArrowUp) {
        loop {
            if menu_state.selected_index == 0 {
                menu_state.selected_index = menu.items.len().saturating_sub(1);
            } else {
                menu_state.selected_index -= 1;
            }

            if menu_state.selected_index < menu.items.len()
                && menu.items[menu_state.selected_index].is_selectable()
            {
                break;
            }

            // Prevent infinite loop if no selectable items
            if menu.selectable_count() == 0 {
                break;
            }
        }
    }

    // Navigate down
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        loop {
            menu_state.selected_index = (menu_state.selected_index + 1) % menu.items.len();

            if menu.items[menu_state.selected_index].is_selectable() {
                break;
            }

            // Prevent infinite loop
            if menu.selectable_count() == 0 {
                break;
            }
        }
    }

    // Handle Enter key
    if keyboard.just_pressed(KeyCode::Enter) {
        if menu_state.selected_index < menu.items.len() {
            let item_type = menu.items[menu_state.selected_index].item_type;
            match item_type {
                MenuItemType::Check => {
                    let item = &mut menu.items[menu_state.selected_index];
                    item.value = if item.value != 0 { 0 } else { 1 };
                }
                MenuItemType::Radio => {
                    // Uncheck all in group, check this one
                    let group = menu.items[menu_state.selected_index].group;
                    for other_item in menu.items.iter_mut() {
                        if other_item.group == group && other_item.item_type == MenuItemType::Radio
                        {
                            other_item.value = 0;
                        }
                    }
                    menu.items[menu_state.selected_index].value = 1;
                }
                _ => {
                    // For Menu type, would trigger callback here
                }
            }
        }
    }

    // Handle Left/Right for sliders
    if menu_state.selected_index < menu.items.len() {
        let item = &mut menu.items[menu_state.selected_index];
        if item.item_type == MenuItemType::Slider {
            if keyboard.just_pressed(KeyCode::ArrowLeft) {
                item.value = (item.value - 1).max(item.min_value);
            }
            if keyboard.just_pressed(KeyCode::ArrowRight) {
                item.value = (item.value + 1).min(item.max_value);
            }
        }
    }

    // ESC to close menu
    if keyboard.just_pressed(KeyCode::Escape) {
        menu_state.visible = false;
        menu_state.active_menu = None;
    }
}

/// Handle mouse hover over menu items
fn handle_menu_hover(
    mut menu_state: ResMut<MenuState>,
    menu_query: Query<&Menu>,
    interaction_query: Query<(&Interaction, &MenuItemIndex), Changed<Interaction>>,
) {
    if !menu_state.visible || menu_state.active_menu.is_none() {
        return;
    }

    let menu_entity = menu_state.active_menu.unwrap();
    let Ok(menu) = menu_query.get(menu_entity) else {
        return;
    };

    // Update selection based on hover
    for (interaction, item_index) in interaction_query.iter() {
        if *interaction == Interaction::Hovered {
            let idx = item_index.0;
            if idx < menu.items.len() && menu.items[idx].is_selectable() {
                menu_state.selected_index = idx;
            }
        }
    }
}

/// Handle mouse clicks on menu items
fn handle_menu_click(
    mut menu_state: ResMut<MenuState>,
    mut menu_query: Query<&mut Menu>,
    interaction_query: Query<(&Interaction, &MenuItemIndex), Changed<Interaction>>,
) {
    if !menu_state.visible || menu_state.active_menu.is_none() {
        return;
    }

    let menu_entity = menu_state.active_menu.unwrap();
    let Ok(mut menu) = menu_query.get_mut(menu_entity) else {
        return;
    };

    // Handle clicks on menu items
    for (interaction, item_index) in interaction_query.iter() {
        if *interaction == Interaction::Pressed {
            let idx = item_index.0;
            if idx >= menu.items.len() {
                continue;
            }

            let item_type = menu.items[idx].item_type;
            match item_type {
                MenuItemType::Check => {
                    let item = &mut menu.items[idx];
                    item.value = if item.value != 0 { 0 } else { 1 };
                }
                MenuItemType::Radio => {
                    // Uncheck all in group, check this one
                    let group = menu.items[idx].group;
                    for other_item in menu.items.iter_mut() {
                        if other_item.group == group && other_item.item_type == MenuItemType::Radio
                        {
                            other_item.value = 0;
                        }
                    }
                    menu.items[idx].value = 1;
                }
                MenuItemType::Slider => {
                    // Start dragging
                    menu_state.dragging_slider = Some(idx);
                }
                MenuItemType::Menu => {
                    // Would trigger callback here
                }
                _ => {}
            }
        }
    }
}

/// Handle slider drag interactions
fn handle_slider_drag(
    mut menu_state: ResMut<MenuState>,
    mut menu_query: Query<&mut Menu>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    slider_query: Query<(&Node, &GlobalTransform, &MenuItemIndex)>,
) {
    // Stop dragging if mouse button released
    if !mouse_button.pressed(MouseButton::Left) {
        menu_state.dragging_slider = None;
        return;
    }

    if !menu_state.visible || menu_state.active_menu.is_none() {
        return;
    }

    let Some(dragging_idx) = menu_state.dragging_slider else {
        return;
    };

    let menu_entity = menu_state.active_menu.unwrap();
    let Ok(mut menu) = menu_query.get_mut(menu_entity) else {
        return;
    };

    if dragging_idx >= menu.items.len() {
        return;
    }

    let item = &menu.items[dragging_idx];
    if item.item_type != MenuItemType::Slider {
        return;
    }

    // Get mouse position
    let Ok(window) = windows.single() else {
        return;
    };
    let Some(cursor_pos) = window.cursor_position() else {
        return;
    };

    // Find the slider UI element
    for (node, global_transform, item_index) in slider_query.iter() {
        if item_index.0 != dragging_idx {
            continue;
        }

        // Get the position and size of the slider element
        let position = global_transform.translation();

        // Calculate approximate width based on node (using computed values)
        let width = match node.width {
            Val::Px(px) => px,
            Val::Percent(pct) => window.width() * pct / 100.0,
            _ => 200.0, // Default fallback
        };

        let left = position.x - width / 2.0;
        let right = position.x + width / 2.0;

        // Calculate new value based on mouse position
        let normalized = ((cursor_pos.x - left) / (right - left)).clamp(0.0, 1.0);
        let range = (item.max_value - item.min_value) as f32;
        let new_value = item.min_value + (normalized * range) as i32;

        menu.items[dragging_idx].value = new_value.clamp(item.min_value, item.max_value);
        break;
    }
}

/// Render menu using Bevy UI
fn render_menu(
    menu_state: Res<MenuState>,
    menu_query: Query<&Menu>,
    mut commands: Commands,
    existing_ui: Query<Entity, With<MenuUI>>,
) {
    // Clean up old UI
    for entity in existing_ui.iter() {
        commands.entity(entity).despawn();
    }

    if !menu_state.visible || menu_state.active_menu.is_none() {
        return;
    }

    let menu_entity = menu_state.active_menu.unwrap();
    let Ok(menu) = menu_query.get(menu_entity) else {
        return;
    };

    // Create menu UI
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            BackgroundColor(menu.bg_color),
            MenuUI,
        ))
        .with_children(|parent| {
            // Title
            parent.spawn((
                Text::new(&menu.title),
                TextFont::from_font_size(32.0),
                TextColor(Color::srgb(0.0, 0.8, 1.0)), // Cyan
            ));

            // Subtitle
            if let Some(ref subtitle) = menu.subtitle {
                parent.spawn((
                    Text::new(subtitle),
                    TextFont::from_font_size(20.0),
                    TextColor(Color::srgb(0.7, 0.7, 0.7)),
                ));
            }

            // Spacer
            parent.spawn(Node {
                height: Val::Px(40.0),
                ..default()
            });

            // Menu items
            for (idx, item) in menu.items.iter().enumerate() {
                let is_selected = idx == menu_state.selected_index;
                let color = if !item.is_selectable() {
                    Color::srgb(0.4, 0.4, 0.4) // Gray for non-selectable
                } else if is_selected {
                    Color::srgb(1.0, 0.8, 0.2) // Orange for selected (like Descent)
                } else {
                    Color::WHITE
                };

                let prefix = if is_selected && item.is_selectable() {
                    "▶ "
                } else {
                    "  "
                };
                let text = format!("{}{}", prefix, item.display_text());

                // Spawn menu item with interaction support
                let mut entity_commands = parent.spawn((
                    Text::new(text),
                    TextFont::from_font_size(24.0),
                    TextColor(color),
                    Node {
                        margin: UiRect::all(Val::Px(5.0)),
                        padding: UiRect::all(Val::Px(5.0)),
                        ..default()
                    },
                    MenuItemIndex(idx),
                ));

                // Add interaction component for selectable items
                if item.is_selectable() {
                    entity_commands.insert(Interaction::None);
                }
            }
        });
}

/// Marker for menu UI entities
#[derive(Component)]
struct MenuUI;

/// Component that stores the index of a menu item in the menu's items list
#[derive(Component)]
struct MenuItemIndex(usize);
