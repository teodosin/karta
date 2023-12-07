use bevy::{prelude::*, ui::FocusPolicy};


// Popup groups
// -----------------------------------------------------------------------------
#[derive(Component, PartialEq, Debug, Clone, Copy)]
pub enum PopupGroup {
    // Context(PopupGroupContext), 
    Context,
    ModalStrong,   
}

// What if groups weren't necessarily mutually exclusive?

// #[derive(Component)]
// struct PopupGroupContext;

// Individual popup components
// -----------------------------------------------------------------------------

#[derive(Component)]
pub struct Popup;

// One-shot Systems 
// -----------------------------------------------------------------------------

pub fn spawn_popup_root(
    mut commands: &mut Commands, 
    menus: Query<(Entity, &PopupGroup), With<Popup>>,
    group: PopupGroup,
    position: Vec2,
    size: Vec2,
) -> Entity {

    // Despawn any menus already spawned from the same group
    clear_popup_group(
        &mut commands,
        &group,
        menus,
    );

    let is_modal = match group {
        PopupGroup::ModalStrong => true,
        _ => false,
    };

    // Handle the popup type
    let popup_root =  commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::rgb(0.0, 0.0, 0.0)),
            transform: Transform::from_translation(Vec3::from((0.0, 0.0, 10000.0))),
            ..Default::default()
        },
        Popup,
        group,
    )).id();

    // Add a background to the popup if it is a modal
    if is_modal {

        println!("Adding modal background for group {:?}", group);
        let popup_bg = commands.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    left: Val::Px(-position.x),
                    top: Val::Px(-position.y),
                    width: Val::Vw(100.0),
                    height: Val::Vh(100.0),
                    
                    ..Default::default()
                },
                //focus_policy: FocusPolicy::Block,
                focus_policy: match group {
                    PopupGroup::ModalStrong => FocusPolicy::Block,
                    _ => FocusPolicy::Pass,
                },
                background_color: BackgroundColor::from(Color::rgba(0.0, 0.0, 0.0, 0.5)),
                transform: Transform::from_translation(Vec3::from((0.0, 0.0, 9999.0))),
                ..Default::default()
            },
        )).id();

        commands.entity(popup_root).push_children(&[popup_bg]);
    }

    popup_root
}

pub fn clear_popup_group(
    commands: &mut Commands,
    target_group: &PopupGroup,
    menus: Query<(Entity, &PopupGroup), With<Popup>>,
) {

    for (menu, group) in menus.iter() {
        println!("Checking group: {:?}", group);
        if *group == *target_group {
            println!("Despawning old menu");
            commands.entity(menu).despawn_recursive();
        }
    }
}

pub fn popup_position_system(
    window: Query<&Window>,
    mut query: Query<(&Node, &mut Style, &GlobalTransform,), With<Popup>>,
) {

    // TODO: Handle multiple windows
    let window = match window.iter().next(){
        None => {return}
        Some(window) => {window}
    };

    let viewport_size = Vec2::new(window.width(), window.height());

    for (popup, mut style, _global_pos) in query.iter_mut() {
        let size = popup.size();
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
