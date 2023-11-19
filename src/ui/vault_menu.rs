// 


use bevy::prelude::*;
use enum_iterator::all;

use crate::{graph::node_types::NodeTypes, actions::{ActionComponent, ActionFactory, node_actions::CreateNodeAction, Action}, input::pointer::InputData};

use super::{popup::{PopupGroup, Popup, spawn_popup_root}, context_menu::ContextMenuButton};
pub struct VaultMenuPlugin;

impl Plugin for VaultMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Update, create_vault_menu)
        ;
    }
}

// TODO: Move key trigger to keymap
fn create_vault_menu(
    mut commands: Commands,
    // event: EventReader<OpenCreateNodeMenuEvent>,
    // event: EventWriter<NodeSpawnedEvent>,
    key: Res<Input<KeyCode>>,
    input_data: Res<InputData>,
    menus: Query<(Entity, &PopupGroup), With<Popup>>,
    window: Query<&Window>,
){
    return

    if !key.just_pressed(KeyCode::Tab) {
        return
    };
    // TODO: Handle multiple windows
    let window = window.single();
    let pos = window.cursor_position();

    let position = match pos {
        None => {
            return
        }
        Some(position) => {
            position
        }
    };
    let global_position = input_data.curr_position;
    let size: Vec2 = Vec2::new(100.0, 100.0);

    let menu_root = spawn_popup_root(
        &mut commands, 
        menus,
        PopupGroup::ModalStrong,
        position,
        size,
    );

    for ntype in all::<NodeTypes>(){
        let button = create_context_menu_button(
            &mut commands,
            ntype,
            global_position,
        );
        commands.entity(menu_root).push_children(&[button]);
    }
}

fn create_context_menu_button<'a>(
    commands: &mut Commands,
    // entity: Entity,
    ntype: NodeTypes,
    position: Vec2,
) -> Entity {
    let label = ntype.to_string();

    let factory: ActionFactory = Box::new(move || {
        let action: Box<dyn Action> = Box::new(CreateNodeAction::new(
            ntype.clone(),
            position.clone(),
        ));
        action
    });


    let button = commands.spawn((
        ActionComponent {
            action: factory,
        },
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
            background_color: super::context_menu::NORMAL_BUTTON.into(),
            ..default()
        },
        ContextMenuButton,
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