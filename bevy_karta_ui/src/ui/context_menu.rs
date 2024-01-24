// The right click menu

use std::any::TypeId;

use bevy::{
    ecs::{system::{EntityCommand, SystemState}, world}, prelude::*, ui::{
        Style, 
        BackgroundColor, 
        Val
    }, utils::HashMap, window::Window
};
use bevy_mod_picking::{prelude::PointerButton, selection::NoDeselect};

use crate::{ events::{nodes::NodeClickEvent, edges::EdgeClickEvent}, input::pointer::InputData};

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
        
        if ev.button != PointerButton::Secondary {
            return
        }

        let window = window.single();
        position = window.cursor_position().unwrap();
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


    // let pin_button = create_context_menu_button(
    //     &mut commands, "Pin".to_string(),
    //     Box::new(|| Box::new(PinToPositionAction::new()))
    // );
    // let unpin_button = create_context_menu_button(
    //     &mut commands, "Unpin".to_string(),
    //     Box::new(|| Box::new(UnpinToPositionAction::new()))
    // );

    // let npath = nodepath.clone();
    // let move_to_context_button = create_context_menu_button(
    //     &mut commands, "Go to Context".to_string(),
    //     Box::new(move || Box::new(MoveToContextAction::new(npath.clone())))
    // );
    // let npath = nodepath.clone();
    // let expand_context_button = create_context_menu_button(
    //     &mut commands, "Expand Context".to_string(),
    //     Box::new(move || Box::new(ExpandContextAction::new(npath.clone())))
    // );
    // let npath = nodepath.clone();
    // let collapse_context_button = create_context_menu_button(
    //     &mut commands, "Collapse Context".to_string(),
    //     Box::new(move || Box::new(CollapseContextAction::new(npath.clone())))
    // );

    // commands.entity(menu_root).push_children(&[pin_button]);
    // commands.entity(menu_root).push_children(&[unpin_button]);
    // commands.entity(menu_root).push_children(&[move_to_context_button]);
    // commands.entity(menu_root).push_children(&[expand_context_button]);
    // commands.entity(menu_root).push_children(&[collapse_context_button]);

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

// fn create_context_menu_button<'a>(
//     commands: &mut Commands,
//     label: String,
//     factory: ActionFactory,
// ) -> Entity {
//     let button = commands.spawn((
//         ActionComponent {
//             action: factory,
//         },
//         ButtonBundle {
//             style: Style {
//                 width: Val::Px(120.0),
//                 height: Val::Px(30.0),
//                 // horizontally center child text
//                 justify_content: JustifyContent::Center,
//                 // vertically center child text
//                 align_items: AlignItems::Center,
//                 ..default()
//             },
//             border_color: BorderColor(Color::BLACK),
//             background_color: NORMAL_BUTTON.into(),
//             ..default()
//         },
//         ContextMenuButton,
//         NoDeselect,
//     ))
//     .with_children(|parent| {
//         parent.spawn((
//             TextBundle::from_section(
//                 label,
//                 TextStyle {
//                     font_size: 16.0,
//                     color: Color::rgb(0.9, 0.9, 0.9),
//                     ..default()
//                 },
//             ),
//         ));
//     }).id();

//     button
// }

// pub fn context_menu_button_system(
//     mut interaction_query: Query<
//         (      
//             &Interaction,
//             &mut BackgroundColor,
//             &ContextMenuButton,
//             &ActionComponent,
//         ),
//         (Changed<Interaction>, With<Button>),
//     >,
//     mut manager: ResMut<ActionManager>, 
// ) {
//     for (interaction, mut color, _mode, factory) in &mut interaction_query {
//         // let mode = mode_query.get(children[0]).unwrap();

//         match *interaction {
//             Interaction::Pressed => {
//                 *color = PRESSED_BUTTON.into();
//                 let action = (factory.action)();
//                 manager.queue_action(action);

//             }
//             Interaction::Hovered => {
//                 *color = HOVERED_BUTTON.into();
//             }
//             Interaction::None => {
//                 *color = NORMAL_BUTTON.into();
//             }
//         }
//     }
// }