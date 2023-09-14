// All that is fixed in place in the foreground
// Excludes the graph and floating windows(?)

use bevy::prelude::*;

use crate::context::KartaVault;
pub struct KartaUiPlugin;

impl Plugin for KartaUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreStartup, default_font_setup)
            .add_systems(PreUpdate, 
                default_font_set.run_if(resource_exists::<FontHandle>()))

            .add_systems(Startup, create_tool_menu)
            .add_systems(Startup, create_context_and_active_bar)

            .add_systems(Update, update_context_label.run_if(resource_changed::<KartaVault>()))
            .add_systems(Update, button_system);
            
    }
}

fn default_font_set(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    font_handle: Res<FontHandle>,
){
    if let Some(font) = fonts.remove(&font_handle.0) {
        fonts.set_untracked(TextStyle::default().font, font);
        commands.remove_resource::<FontHandle>();
    }
}

fn default_font_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

) {
    let font = asset_server.load("fonts/Roboto/Roboto-Medium.ttf");
    commands.insert_resource(FontHandle(font));
}

#[derive(Resource)]
struct FontHandle(Handle<Font>);

fn create_tool_menu(
    mut commands: Commands,
){
    commands.spawn(
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(100.0),
                align_items: AlignItems::Center,
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                margin: UiRect {
                    left: Val::Px(20.0),
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                },
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
            width: Val::Px(100.0),
            height: Val::Px(30.0),
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
                font_size: 16.0,
                color: Color::rgb(0.9, 0.9, 0.9),
                ..default()
            },
        ));
    });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct ContextLabel;

#[derive(Component)]
struct ActiveLabel;

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
            }
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
        }
    }
}

fn create_context_and_active_bar(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
){
    commands.spawn(
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(600.0),
                align_items: AlignItems::Start,
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                margin: UiRect {
                    left: Val::Px(20.0),
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                },
                ..default()
            },
            ..default()
        })
            .with_children(|parent| {
                parent.spawn((TextBundle::from_section(
                    "Context",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ContextLabel 
                ));
                parent.spawn((TextBundle::from_section(
                    "Active",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ActiveLabel 
                ));
            });
}

fn update_context_label(
    mut query: Query<&mut Text, With<ContextLabel>>,
    vault: Res<KartaVault>,
){
    for mut text in &mut query.iter_mut() {
        text.sections[0].value = vault.current_context.clone();
    }
}

