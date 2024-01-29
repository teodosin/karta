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
    mut mouse_event: EventReader<NodeClickEvent>, 
    view_nodes: Query<&GraphViewNode>,
    mut commands: Commands,
    window: Query<&Window>,
    mut menus: Query<(Entity, &PopupGroup), With<Popup>>,
    e_sys: Res<ContextEntitySystems>,
    mut menu_event: EventWriter<ContextMenuSpawnEvent>,
) {
    
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
    let target = match view_nodes.get(viewtarget){
        Ok(node) => node.get_target(),
        Err(_) => return,
    };

    let window = window.single();
    let position = window.cursor_position().unwrap();
    

    println!("Spawning context menu");

    // TODO: Handle multiple windows

    let size: Vec2 = Vec2::new(120.0, 100.0);

    // Get a popup root
    let menu_root = spawn_popup_root(&mut commands, &mut menus, PopupGroup::Context, position, size);
    
    for sys in e_sys.get() {
        let button = create_context_menu_button(
            &mut commands, sys.name().clone(), sys.command()
        );
        commands.entity(menu_root).push_children(&[button]);
    }

    menu_event.send(ContextMenuSpawnEvent {
        target,
        menu_root,
    });


}

#[derive(Event)]
pub struct ContextMenuSpawnEvent {
    pub target: Entity,
    pub menu_root: Entity,
}

pub fn add_component_systems_to_context_menu(
    mut world: &mut World,
){
    let mut system_state: SystemState<
        EventReader<ContextMenuSpawnEvent>,
    > = SystemState::new(&mut world);

    let mut menu_event = system_state.get_mut(&mut world);

    if menu_event.is_empty() {
        return;
    }
    
    println!("running systemsssss {}", menu_event.len());
    
    let (target, menu_root) = match menu_event.read().next() {
        Some(ev) => (ev.target, ev.menu_root),
        None => return,
    };
    menu_event.clear();

    // Make sure both still exist before proceeding
    if world.get_entity(target).is_none() || world.get_entity(menu_root).is_none() {
        return;
    }

    let target_components = {
        let mut target_components = Vec::new();
        let component_infos = world.inspect_entity(target);
        for cmp in component_infos.iter() {
            let id = cmp.type_id().unwrap();
            target_components.push(id);
        }
        target_components
    };
    let mut system_state: SystemState<
        Res<ContextComponentSystems>,
    > = SystemState::new(&mut world);

    let c_sys = system_state.get_mut(&mut world);
    let c_sys = c_sys.clone();

    println!("Length of target components: {}", target_components.len());

    for id in target_components.iter() {
        let cmds = c_sys.get_by_id(*id);

        match cmds {
            Some(cmds) => {
                println!("Length of commands: {}", cmds.len());
                for systems in cmds {
                    let system_id = systems.command();
                    let label = systems.name();
                    let button = world.spawn((
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
                    }).id();
                    world.entity_mut(menu_root).push_children(&[button]);
                }
            }
            None => continue,
        }
    }
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
pub struct ButtonSystem {
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
