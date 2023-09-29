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

use crate::graph::nodes::NodeClickEvent;

#[derive(Component)]
pub struct PopupMenu;

#[derive(Component)]
pub struct PopupMenuButton;

pub fn despawn_context_menus(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    menus: Query<Entity, With<PopupMenu>>,
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
    mouse: Res<Input<MouseButton>>,
    menus: Query<Entity, With<PopupMenu>>,
    window: Query<&Window>,
){

    // Despawn any menus already spawned
    let inputs = [MouseButton::Left, MouseButton::Middle, MouseButton::Right];
    if mouse.any_pressed(inputs) {
        for menu in menus.iter() {
            commands.entity(menu).despawn_recursive();
        }
    }

    if mouse_event.is_empty() {
        return
    }

    if mouse_event.iter().next().unwrap().button != MouseButton::Right {
        return
    }

    println!("Spawning context menu");

    let window = window.single();

    let position: Vec2 = window.cursor_position().unwrap();

    commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        PopupMenu,
    ))
    .with_children(
        |parent| {
            create_context_menu_button(parent, "Pin".to_string());
        }
    );
}

const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn create_context_menu_button<'a>(
    parent: &mut ChildBuilder<'_, '_, '_>,
    action: String,
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
    });
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