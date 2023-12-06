// 


use bevy::prelude::*;
use enum_iterator::all;

use crate::{graph::node_types::NodeTypes, actions::{ActionComponent, ActionFactory, node_actions::CreateNodeAction, Action}, input::pointer::InputData, vault::CurrentVault};

use super::{popup::{PopupGroup, Popup, spawn_popup_root}, context_menu::ContextMenuButton};
pub struct VaultMenuPlugin;

impl Plugin for VaultMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnVaultMenu>()
            .add_systems(Update, create_vault_menu)
        ;
    }
}

// Run trigger condition
fn no_vault_set(
    vault: Res<CurrentVault>,
) -> bool {
    vault.vault.is_none()
}

#[derive(Event)]
pub struct SpawnVaultMenu;

// TODO: Move key trigger to keymap
fn create_vault_menu(
    mut commands: Commands,
    menus: Query<(Entity, &PopupGroup), With<Popup>>,
    window: Query<&Window>,
    mut evs: EventReader<SpawnVaultMenu>,
){

    for _ev in evs.read(){
        // TODO: Handle multiple windows
        let window = window.single();
    
        let size: Vec2 = Vec2::new(400.0, 600.0);
        let pos = Vec2::new(
            window.height() + size.x / 2.0,
            window.width() + size.y / 2.0,
        );
    
        let menu_root = spawn_popup_root(
            &mut commands, 
            menus,
            PopupGroup::ModalStrong,
            pos,
            size,
        );

        break;
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