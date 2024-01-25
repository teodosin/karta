// The right click menu

use std::{any::TypeId, cmp};

use bevy::{
    ecs::{system::{EntityCommand, SystemState}, world}, prelude::*, ui::{
        Style, 
        BackgroundColor, 
        Val
    }, utils::HashMap, window::Window
};
use bevy_mod_picking::{events::{Click, Pointer}, prelude::{On, PointerButton}, selection::NoDeselect};

use crate::{ events::{nodes::NodeClickEvent, edges::EdgeClickEvent}, input::pointer::InputData, prelude::context_commands::{ComponentCommands, CustomCommand}};

use super::popup::*;

#[derive(Component)]
pub struct ContextMenuButton;

// TODO: Revisit the logic for when a popup should be despawned, exactly
pub fn despawn_context_menus_on_any_click(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    menus: Query<(Entity, &PopupGroup), With<Popup>>,
) {
    let input = [MouseButton::Left, MouseButton::Right, MouseButton::Middle];
    if mouse.any_just_released(input) {
    // if mouse.is_changed() {
        for (menu, group) in menus.iter() {
            match group {
                PopupGroup::Context => {
                    commands.entity(menu).despawn_recursive();
                },
                _ => {},
            }
        }
    }
}

/// System for spawning the context menu on right click of a node. 
pub fn spawn_node_context_menu(
    mut world: &mut World,
){
    let position: Vec2;
    let target: Entity;
    let mut is_right_click: bool = false;

    {    
        let mut system_state: SystemState<(
            EventReader<NodeClickEvent>,
            Query<&Window>
        )> = SystemState::new(&mut world);

        let (mut mouse_event, window) = system_state.get_mut(&mut world);
            
        if mouse_event.is_empty() {
            return
        }

        
        let ev = mouse_event.read().next();
        println!("ev: {:?}", ev);

        let ev = match ev {
            Some(ev) => ev,
            None => return,
        };

        println!("Mouse event: {:?}", ev);
        
        if ev.button == PointerButton::Secondary {
            is_right_click = true;
        }

        target = ev.target.unwrap();
        let window = window.single();
        position = window.cursor_position().unwrap();
    }

    if !is_right_click {
        return
    }

  

    println!("Spawning context menu");

    // TODO: Handle multiple windows

    let size: Vec2 = Vec2::new(120.0, 100.0);

    // Get a popup root
    let menu_root = spawn_popup_root(
        &mut world,
        PopupGroup::Context,
        position,
        size,
    );

    println!("Menu root: {:?}", menu_root);
    let mut buttons: Vec<(Entity, &CustomCommand)> = Vec::new();


    {        
        let commands = world.get_resource::<ComponentCommands>().unwrap();
        let target_components = world.inspect_entity(target);

        for cmp in target_components.iter() {
            let id = cmp.type_id().unwrap();
            let cmds = commands.get_by_id(id);
            
            match cmds {
                Some(cmds) => {
                    for command in cmds {
                        buttons.push((menu_root, command));
                    }
                },
                None => continue,
            }
        }
    }

    // let pin_button = create_context_menu_button(
    //     &mut commands, "Pin".to_string(),
    //     Box::new(|| Box::new(PinToPositionAction::new()))
    // );
    // commands.entity(menu_root).push_children(&[pin_button]);
}

pub fn spawn_edge_context_menu(
    mut world: &mut World,
){
    let mut system_state: SystemState<(
        Commands,
        EventReader<EdgeClickEvent>,
        Query<(Entity, &PopupGroup), With<Popup>>,
        Query<&Window>
    )> = SystemState::new(&mut world);

    let (mut commands, mut mouse_event, menus, window) = system_state.get_mut(&mut world);
    
    if mouse_event.is_empty() {
        return
    }
    
    // let target = mouse_event.read().next().unwrap().target.unwrap();
    let button = mouse_event.read().next().unwrap().button;
    
    if button != PointerButton::Secondary {
        return
    }    

    println!("Spawning context menu");

    // TODO: Handle multiple windows
    let window = window.single();

    let position: Vec2 = window.cursor_position().unwrap();
    let size: Vec2 = Vec2::new(120.0, 100.0);

    // Get a popup root
    let menu_root = spawn_popup_root( 
        &mut world,
        PopupGroup::Context,
        position,
        size,
    );

    // let delete_button = create_context_menu_button(
    //     &mut commands, "Delete edge".to_string(),
    //     Box::new(|| Box::new(DeleteEdgeAction::new()))
    // );

    // commands.entity(menu_root).push_children(&[delete_button]);


}

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

#[derive(Component)]
struct ButtonCommand {
    entity: Entity, 
    cmd: Box<dyn EntityCommand + Sync + 'static>,
}

fn create_context_menu_button<'a>(
    commands: &mut World,
    label: String,
    entity: Entity,
    command: Box<dyn EntityCommand + Sync + 'static>,
) -> Entity {
    let button = commands.spawn((
        ButtonCommand {
            entity,
            cmd: command,
        },
        ButtonBundle {
            style: Style {
                width: Val::Px(120.0),
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
        ContextMenuButton,
        NoDeselect,
    ))
    .with_children(|parent| {
        parent.spawn((
            TextBundle::from_section(
                label,
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

/// Todo: turn into normal system, get rid of mutable world access.
/// Defeats the purpose of using Commands. 
pub fn context_menu_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (      
            &Interaction,
            &mut BackgroundColor,
            &ContextMenuButton,
            &ButtonCommand,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, _mode, command) in &mut interaction_query {
        // let mode = mode_query.get(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                commands.entity(command.entity).add(command.cmd);
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