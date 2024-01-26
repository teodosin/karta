// The right click menu

use std::{any::TypeId, cmp};

use bevy::{
    ecs::system::{EntityCommand, SystemId, SystemState},
    prelude::*,
    ui::{BackgroundColor, Style, Val},
    window::Window,
};
use bevy_mod_picking::{
    prelude::{On, PointerButton},
    selection::NoDeselect,
};

use crate::{
    events::{edges::EdgeClickEvent, node_events::NodeClickEvent}, prelude::context_commands::{ContextComponentSystems, ContextEntitySystems, ContextSystem}, ui::nodes::GraphViewNode
};

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
                }
                _ => {}
            }
        }
    }
}

/// System for spawning the context menu on right click of a node.
pub fn spawn_node_context_menu(
    mut world: &mut World
) {

    // Get event information to determine target and button
    // Get the list of components on the target entity

    let mut system_state: SystemState<(
        EventReader<NodeClickEvent>, 
        Query<&GraphViewNode>,
    )> = SystemState::new(&mut world);

    let (mut mouse_event, view_nodes) = system_state.get_mut(&mut world);
    
    if mouse_event.is_empty() {
        return;
    }

    let ev = mouse_event.read().next();

    let ev = match ev {
        Some(ev) => ev,
        None => return,
    };

    if ev.button != PointerButton::Secondary {
        return
    }
    
    let viewtarget = ev.target.unwrap();
    let target = view_nodes.get(viewtarget).unwrap().get_target();

    let target_components = {
        let mut target_components = Vec::new();
        let component_infos = world.inspect_entity(target);
        for cmp in component_infos.iter() {
            let id = cmp.type_id().unwrap();
            target_components.push(id);
        }
        target_components
    };
    // --------------------------------

    let mut system_state: SystemState<(
        Commands,
        Query<&Window>,
        Query<(Entity, &PopupGroup), With<Popup>>,
        Res<ContextEntitySystems>,
        Res<ContextComponentSystems>,
    )> = SystemState::new(&mut world);

    let (mut commands, window, mut menus, e_sys, c_sys) = system_state.get_mut(&mut world);

    let window = window.single();
    let position = window.cursor_position().unwrap();
    

    println!("Spawning context menu");

    // TODO: Handle multiple windows

    let size: Vec2 = Vec2::new(120.0, 100.0);

    // Get a popup root
    let menu_root = spawn_popup_root(&mut commands, &mut menus, PopupGroup::Context, position, size);
    
    

    for id in target_components.iter() {
        let cmds = c_sys.get_by_id(*id);

        match cmds {
            Some(cmds) => {
                for systems in cmds {
                    let button = create_context_menu_button(
                        &mut commands, systems.name().clone(), systems.command()
                    );
                    commands.entity(menu_root).push_children(&[button]);
                }
            }
            None => continue,
        }
    }
    

    // let pin_button = create_context_menu_button(
    //     &mut commands, "Pin".to_string(),
    //     Box::new(|| Box::new(PinToPositionAction::new()))
    // );
    // commands.entity(menu_root).push_children(&[pin_button]);
}

pub fn spawn_edge_context_menu(mut world: &mut World) {
    let mut system_state: SystemState<(
        Commands,
        EventReader<EdgeClickEvent>,
        Query<(Entity, &PopupGroup), With<Popup>>,
        Query<&Window>,
    )> = SystemState::new(&mut world);

    let (mut commands, mut mouse_event, menus, window) = system_state.get_mut(&mut world);

    if mouse_event.is_empty() {
        return;
    }

    // let target = mouse_event.read().next().unwrap().target.unwrap();
    let button = mouse_event.read().next().unwrap().button;

    if button != PointerButton::Secondary {
        return;
    }

    println!("Spawning context menu");

    // TODO: Handle multiple windows
    let window = window.single();

    let position: Vec2 = window.cursor_position().unwrap();
    let size: Vec2 = Vec2::new(120.0, 100.0);

    // Get a popup root
    let menu_root = spawn_popup_root(&mut commands, &menus, PopupGroup::Context, position, size);

    // let delete_button = create_context_menu_button(
    //     &mut commands, "Delete edge".to_string(),
    //     Box::new(|| Box::new(DeleteEdgeAction::new()))
    // );

    // commands.entity(menu_root).push_children(&[delete_button]);
}

pub const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
pub const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
pub const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

pub trait CustomEntityCommand: EntityCommand + Sync + 'static {
    fn execute(&self, commands: &mut Commands);
}
#[derive(Component)]
struct ButtonSystem {
    system: SystemId,
}

impl ButtonSystem {
    fn id(&self) -> SystemId {
         self.system
    }
}

fn create_context_menu_button<'a>(
    commands: &mut Commands,
    label: String,
    system_id: SystemId,
) -> Entity {
    let button = commands
        .spawn((
            ButtonSystem {
                system: system_id,
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
            parent.spawn((TextBundle::from_section(
                label,
                TextStyle {
                    font_size: 16.0,
                    color: Color::rgb(0.9, 0.9, 0.9),
                    ..default()
                },
            ),));
        })
        .id();

    button
}

pub fn context_menu_button_system(
    mut commands: Commands,
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &ContextMenuButton,
            &ButtonSystem,
        ),
        (Changed<Interaction>, With<Button>),
    >,
) {
    for (interaction, mut color, _mode, command) in &mut interaction_query {
        // let mode = mode_query.get(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();
                commands.run_system(command.id());
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
