// All that is fixed in place in the foreground
// Excludes the graph and floating windows(?)

use bevy::prelude::*;

use bevy_prototype_lyon::prelude::*;

use crate::{
    graph::context::{CurrentContext, update_context},
    events::nodes::*};

use self::{
    context_menu::popup_menu_button_system, 
    mode_menu::{create_mode_menu, mode_button_system, update_active_mode_highlight}, 
    nodes::{add_node_ui, handle_outline_hover, outlines_pulse},
};

mod context_menu;
mod mode_menu;
pub(crate) mod nodes;
mod edges;

pub struct KartaUiPlugin;

impl Plugin for KartaUiPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(ShapePlugin)

            .add_systems(PreStartup, default_font_setup)
            .add_systems(PreUpdate, 
                default_font_set.run_if(resource_exists::<FontHandle>()))

                
            .add_systems(Startup, gizmo_settings)
            .add_systems(Startup, create_mode_menu)
            .add_systems(Startup, create_context_and_active_bar)
            
            .add_systems(Update, update_context_label.run_if(resource_changed::<CurrentContext>()))
            
            .add_systems(Update, mode_button_system)
            .add_systems(Update, update_active_mode_highlight.after(mode_button_system))
            
            .add_systems(Update, popup_menu_button_system)
            
            .add_systems(Update, context_menu::despawn_context_menus)
            .add_systems(
                Update, 
                context_menu::spawn_context_menu.run_if(on_event::<NodeClickEvent>())
            )

            // .add_systems(PostUpdate, add_node_ui
            //     .after(update_context)
            //     .run_if(resource_changed::<CurrentContext>())
            // )
            
            .add_systems(PostUpdate, handle_outline_hover)
            .add_systems(PostUpdate, outlines_pulse)
            
        ;
    }
}

fn default_font_set(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    font_handle: Res<FontHandle>,
){
    if let Some(font) = fonts.remove(&font_handle.0) {
        fonts.set_untracked(TextStyle::default().font, font);
        commands.remove_resource::<FontHandle>();
    }
}

fn default_font_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

) {
    let font = asset_server.load("fonts/Roboto/Roboto-Medium.ttf");
    commands.insert_resource(FontHandle(font));
}

#[derive(Resource)]
struct FontHandle(Handle<Font>);

fn gizmo_settings(
    mut gizmo: ResMut<GizmoConfig>,
){
    gizmo.depth_bias = 1.0;
}


#[derive(Component)]
struct ContextLabel;

#[derive(Component)]
struct ActiveLabel;

fn create_context_and_active_bar(
    mut commands: Commands,
    _asset_server: Res<AssetServer>,
){
    commands.spawn(
        NodeBundle {
            style: Style {
                flex_direction: FlexDirection::Column,
                width: Val::Px(600.0),
                align_items: AlignItems::Start,
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                margin: UiRect {
                    left: Val::Px(20.0),
                    right: Val::Px(20.0),
                    top: Val::Px(20.0),
                    bottom: Val::Px(20.0),
                },
                ..default()
            },
            ..default()
        })
            .with_children(|parent| {
                parent.spawn((TextBundle::from_section(
                    "Context",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ContextLabel 
                ));
                parent.spawn((TextBundle::from_section(
                    "Active",
                    TextStyle {
                        font_size: 16.0,
                        color: Color::rgb(0.9, 0.9, 0.9),
                        ..default()
                    },
                ),
                ActiveLabel 
                ));
            });
}

fn update_context_label(
    mut query: Query<&mut Text, With<ContextLabel>>,
    vault: Res<CurrentContext>,
){
    for mut text in &mut query.iter_mut() {
        text.sections[0].value = vault.current_context.clone();
    }
}

