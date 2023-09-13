// All that is fixed in place in the foreground
// Excludes the graph and floating windows(?)

use bevy::prelude::*;
pub struct KartaUiPlugin;

impl Plugin for KartaUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, create_tool_menu);
    }
}

fn create_tool_menu(
    mut commands: Commands,
){
    commands.spawn(
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(50.0),
                align_items: AlignItems::Center,
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        }
    )
    .with_children(|parent| {
        create_tool_menu_button(parent, "Arrange".to_string());
        create_tool_menu_button(parent, "Select".to_string());
    });



}

fn create_tool_menu_button<'a>(
    parent: &mut ChildBuilder<'_, '_, '_>,
    tool: String,
) {
    parent.spawn(ButtonBundle {
        style: Style {
            width: Val::Px(50.0),
            height: Val::Px(20.0),
            border: UiRect::all(Val::Px(5.0)),
            // horizontally center child text
            justify_content: JustifyContent::Center,
            // vertically center child text
            align_items: AlignItems::Center,
            ..default()
        },
        border_color: BorderColor(Color::BLACK),
        background_color: NORMAL_BUTTON.into(),
        ..default()
    })
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            tool,
            TextStyle {
                font_size: 12.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
