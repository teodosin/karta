// 


use bevy::prelude::*;
use native_dialog::FileDialog;

use crate::vault::{CurrentVault, VaultOfVaults, KartaVault};

use super::{popup::{PopupGroup, Popup,}, context_menu::{HOVERED_BUTTON, NORMAL_BUTTON, PRESSED_BUTTON}};
pub struct VaultMenuPlugin;

impl Plugin for VaultMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<SpawnVaultMenu>()
            .add_systems(Update, create_vault_menu)
            .add_systems(Update, vault_menu_button_system)
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

#[derive(Component)]
pub struct VaultMenu;

// TODO: Move key trigger to keymap
fn create_vault_menu(
    mut commands: Commands,
    _menus: Query<(Entity, &PopupGroup), With<Popup>>,
    mut evs: EventReader<SpawnVaultMenu>,
){

    for _ev in evs.read(){
        // TODO: Handle multiple windows    
        let size: Vec2 = Vec2::new(500.0, 600.0);
    
        let menu_root = commands.spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Relative,
                    flex_direction: FlexDirection::Column,
                    align_self: AlignSelf::Center,
                    justify_self: JustifySelf::Center,
                    justify_content: JustifyContent::SpaceAround,
                    width: Val::Px(size.x),
                    height: Val::Px(size.y),
                    
                    ..Default::default()
                },
                background_color: BackgroundColor::from(Color::rgb(0.0, 0.0, 0.0)),
                transform: Transform::from_translation(Vec3::from((0.0, 0.0, 10000.0))),
                ..Default::default()
            },
            VaultMenu,
        )).id();

        let copy = include_str!("../../assets/copy/welcome_and_vault.txt");

        let textblock = commands.spawn(
            TextBundle {
                text: Text {
                    sections: vec![
                        TextSection {
                            value: copy.to_string(),
                            style: TextStyle {
                                ..default()
                            },
                        },
                    ],
                    ..default()
                },
                style: Style {
                    margin: UiRect::all(Val::Px(50.0)),
                    ..default()
                },
                ..default()
            }
        ).id();

        let vault_button = create_vault_menu_button(&mut commands);

        commands.entity(menu_root).push_children(&[textblock]);
        commands.entity(menu_root).push_children(&[vault_button]);

        break;
    }

}

#[derive(Component)]
pub struct VaultMenuButton;

fn create_vault_menu_button<'a>(
    commands: &mut Commands,
    // entity: Entity,
) -> Entity {
    let label = String::from("Create Vault");


    let button = commands.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(200.0),
                height: Val::Px(60.0),
                align_self: AlignSelf::Center,
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
        VaultMenuButton,
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

pub fn vault_menu_button_system(
    mut interaction_query: Query<
        (      
            &Interaction,
            &mut BackgroundColor,
            &VaultMenuButton,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut vaults: ResMut<VaultOfVaults>,
    mut curvault: ResMut<CurrentVault>,

    _: NonSend<bevy::winit::WinitWindows>,
) {
    for (interaction, mut color, _mode) in &mut interaction_query {
        // let mode = mode_query.get(children[0]).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *color = PRESSED_BUTTON.into();

                let folder = FileDialog::new()
                    .set_title("Select Karta Vault Location")
                    .show_open_single_dir();

                let folder = match folder {
                    Ok(folder) => folder,
                    Err(_) => None,
                };

                match folder {
                    Some(folder) => {
                        if !folder.is_dir() {
                            println!("Not a folder");
                            return
                        }

                        let vault = KartaVault::new(folder);
                        vaults.add_vault(vault.clone());
                        curvault.set_vault(vault);
                    },
                    None => {
                        println!("No folder selected");
                    }
                }

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