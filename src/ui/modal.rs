use bevy::prelude::*;
use bevy_mod_picking::prelude::PointerButton;

use crate::events::nodes::NodeClickEvent;

// Modal groups
// -----------------------------------------------------------------------------
#[derive(Component, PartialEq)]
pub enum ModalGroup {
    // Context(ModalGroupContext), 
    Context,
    Input,   
}

// What if groups weren't necessarily mutually exclusive?

// #[derive(Component)]
// struct ModalGroupContext;

// Individual modal components
// -----------------------------------------------------------------------------

#[derive(Component)]
pub struct Modal;

// One-shot Systems 
// -----------------------------------------------------------------------------

pub fn spawn_modal_root(
    mut commands: &mut Commands, 
    menus: Query<(Entity, &ModalGroup), With<Modal>>,
    group: ModalGroup,
    position: Vec2,
) -> Entity {

    // Despawn any menus already spawned from the same group
    clear_modal_group(
        &mut commands,
        group,
        menus,
    );

    // Handle the modal type
    let modal_root =  commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(100.0),
                height: Val::Px(100.0),
                
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::rgb(0.0, 0.0, 0.0)),
            ..Default::default()
        },
        Modal,
    )).id();

    modal_root
}

pub fn clear_modal_group(
    commands: &mut Commands,
    target_group: ModalGroup,
    menus: Query<(Entity, &ModalGroup), With<Modal>>,
) {
    for (menu, group) in menus.iter() {
        if *group == target_group {
            commands.entity(menu).despawn_recursive();
        }
    }
}

pub fn modal_position_system(
    window: Query<&Window>,
    mut query: Query<(&Node, &mut Style, &GlobalTransform,), With<Modal>>,
) {

    // TODO: Handle multiple windows
    let window = window.single();

    let viewport_size = Vec2::new(window.width(), window.height());

    for (modal, mut style, global_pos) in query.iter_mut() {
        let size = modal.size();
        let window_size = Vec2::new(window.width(), window.height());

        let left = style.left.resolve(1., viewport_size).unwrap();
        if left + size.x > window_size.x {
            style.left = Val::Px(left - size.x);
        }

        let top = style.top.resolve(1., viewport_size).unwrap();
        if top + size.y > window_size.y {
            style.top = Val::Px(top - size.y);
        }
    }
}
