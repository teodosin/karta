// All that is fixed in place in the foreground
// Excludes the graph and floating windows(?)

use bevy::prelude::*;

use crate::{graph::context::CurrentContext, modes::KartaModeState};
pub struct KartaUiPlugin;

impl Plugin for KartaUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(PreStartup, default_font_setup)
            .add_systems(PreUpdate, 
                default_font_set.run_if(resource_exists::<FontHandle>()))

            .add_systems(Startup, gizmo_settings)
            .add_systems(Startup, create_mode_menu)
            .add_systems(Startup, create_context_and_active_bar)

            .add_systems(Update, update_context_label.run_if(resource_changed::<CurrentContext>()))
            .add_systems(Update, button_system)
            .add_systems(Update, update_active_mode_highlight.after(button_system))
        ;
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

fn gizmo_settings(
    mut gizmo: ResMut<GizmoConfig>,
){
    gizmo.depth_bias = 1.0;
}

// Marker components for the mode buttons and their labels
#[derive(Component)]
struct ModeButton {
    mode: KartaModeState,
}

#[derive(Component)]
struct ModeButtonLabel {
    mode: KartaModeState,
}

fn create_mode_menu(
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
        create_mode_menu_button(parent, KartaModeState::Arrange);
        create_mode_menu_button(parent, KartaModeState::Select);
        create_mode_menu_button(parent, KartaModeState::Edges);
        create_mode_menu_button(parent, KartaModeState::State);

    });



}

fn create_mode_menu_button<'a>(
    parent: &mut ChildBuilder<'_, '_, '_>,
    mode: KartaModeState,
) {
    parent.spawn((
        ButtonBundle {
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
        },
        ModeButton {
            mode: mode.clone(),
        },
    ))
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                mode.to_string(),
                TextStyle {
                    font_size: 16.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),
            ModeButtonLabel {
                mode: mode,
            },
        ));
    });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const ACTIVE_BUTTON: Color = Color::rgb(0.8, 0.6, 0.23);

#[derive(Component)]
struct ContextLabel;

#[derive(Component)]
struct ActiveLabel;

fn button_system(
    mut interaction_query: Query<
        (      
            &Interaction,
            &mut BackgroundColor,
            &ModeButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    // text_query: Query<&Text, With<ModeButton>>,
    mut next_state: ResMut<NextState<KartaModeState>>,
) {
    for (interaction, mut color, mode) in &mut interaction_query {
        // let mode = mode_query.get(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                
                match mode.mode {
                    KartaModeState::Arrange => {
                        println!("Arrange mode");
                    }
                    KartaModeState::Select => {
                        println!("Select mode");
                    }
                    KartaModeState::Edges => {
                        println!("Edges mode");
                    }
                    KartaModeState::State => {
                        println!("Context mode");
                    }
                }
                next_state.set(mode.mode.clone());
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

fn update_active_mode_highlight(
    mut active_query: Query<
    (&mut BackgroundColor, &ModeButton, &Interaction), With<Button>
    >,
    cur_state: Res<State<KartaModeState>>,

){
    for (mut color, mode, interaction) in &mut active_query.iter_mut() {
        if mode.mode == **cur_state {
            *color = ACTIVE_BUTTON.into();
        } else {
            match interaction {
                Interaction::Hovered => {
                    *color = HOVERED_BUTTON.into();
                }
                Interaction::None => {
                    *color = NORMAL_BUTTON.into();
                }
                _ => {}
            }
        }
    }
}

fn create_context_and_active_bar(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
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
    vault: Res<CurrentContext>,
){
    for mut text in &mut query.iter_mut() {
        text.sections[0].value = vault.current_context.clone();
    }
}

