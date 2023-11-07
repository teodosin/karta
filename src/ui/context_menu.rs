// The right click menu

use bevy::{
    prelude::*, 
    ui::{
        Style, 
        PositionType, 
        BackgroundColor, 
        Val
    }, 
    window::Window
};
use bevy_mod_picking::prelude::PointerButton;

use crate::events::nodes::NodeClickEvent;

use super::modal::*;

enum ContextMenuButtons {
    Pin,
    GoToContext,
}

#[derive(Component)]
pub struct PopupMenuButton;

pub fn despawn_context_menus(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    menus: Query<Entity, With<Modal>>,
) {
    if mouse.is_changed() {
        for menu in menus.iter() {
            commands.entity(menu).despawn_recursive();
        }
    }
}

pub fn spawn_context_menu(
    mut commands: Commands,
    mut mouse_event: EventReader<NodeClickEvent>,
    menus: Query<(Entity, &ModalGroup), With<Modal>>,
    window: Query<&Window>,
){

    let inputs = [PointerButton::Primary, PointerButton::Secondary, PointerButton::Middle];
    
    if mouse_event.is_empty() {
        return
    }
    
    let button = mouse_event.iter().next().unwrap().button;
    
    if button != PointerButton::Secondary {
        return
    }

    println!("Spawning context menu");

    // TODO: Handle multiple windows
    let window = window.single();

    let position: Vec2 = window.cursor_position().unwrap();

    // Get a modal root
    let menu_root = spawn_modal_root(
        &mut commands, 
        menus,
        ModalGroup::Context,
        position,
    );

    let pin_button = create_context_menu_button(&mut commands, "Pin".to_string());
    let move_to_context_button = create_context_menu_button(&mut commands, "Go to Context".to_string());

    commands.entity(menu_root).push_children(&[pin_button]);
    commands.entity(menu_root).push_children(&[move_to_context_button]);

}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn create_context_menu_button<'a>(
    commands: &mut Commands,
    action: String,
) -> Entity {
    let button = commands.spawn((
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
        PopupMenuButton,
    ))
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                action,
                TextStyle {
                    font_size: 16.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),
        ));
    }).id();

    button
}

pub fn popup_menu_button_system(
    mut interaction_query: Query<
        (      
            &Interaction,
            &mut BackgroundColor,
            &PopupMenuButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, mode) in &mut interaction_query {
        // let mode = mode_query.get(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                println!("Pinned suckaa!!");

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