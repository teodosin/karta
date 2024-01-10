use bevy::prelude::*;
use bevy_mod_picking::selection::NoDeselect;

use crate::bevy_overlay_graph::{modes::KartaModeState, events::nodes::*};


// Marker components for the mode buttons and their labels
#[derive(Component)]
pub struct ModeButton {
    mode: KartaModeState,
}


pub fn create_mode_menu(
    mut commands: Commands,
){
    commands.spawn(
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                width: Val::Px(60.0),
                align_items: AlignItems::Center,
                align_self: AlignSelf::Center,
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
        create_mode_menu_button(parent, KartaModeState::Move);
        create_mode_menu_button(parent, KartaModeState::Edges);
        create_mode_menu_button(parent, KartaModeState::State);
        create_mode_menu_button(parent, KartaModeState::Play);

    });



}

pub fn create_mode_menu_button<'a>(
    parent: &mut ChildBuilder<'_, '_, '_>,
    mode: KartaModeState,
) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(60.0),
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
            NoDeselect,
            TextBundle::from_section(
                mode.to_string(),
                TextStyle {
                    font_size: 16.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),
        ));
    });
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);
const ACTIVE_BUTTON: Color = Color::rgb(0.8, 0.6, 0.23);

pub fn mode_button_system(
    mut interaction_query: Query<
        (      
            &Interaction,
            &mut BackgroundColor,
            &ModeButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,

    // We have to manually clear these events because of the silly way 
    // that bevy has changed the way that events are handled
    mut event_click: EventReader<NodeClickEvent>,
    mut event_press: EventReader<NodePressedEvent>,
    mut event_hover: EventReader<NodeHoverEvent>,

    // text_query: Query<&Text, With<ModeButton>>,
    mut next_state: ResMut<NextState<KartaModeState>>,
) {
    for (interaction, mut color, mode) in &mut interaction_query {
        // let mode = mode_query.get(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                
                match mode.mode {
                    KartaModeState::Move => {
                        println!("Move mode");
                    }
                    KartaModeState::Edges => {
                        println!("Edges mode");
                    }
                    KartaModeState::State => {
                        println!("Context mode");
                    }
                    KartaModeState::Play => {
                        println!("Play mode");
                    }
                }

                event_click.clear();
                event_press.clear();
                event_hover.clear();
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

pub fn update_active_mode_highlight(
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