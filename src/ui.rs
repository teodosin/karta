// All that is fixed in place in the foreground
// Excludes the graph and floating windows(?)

use bevy::{prelude::*, ui::FocusPolicy, render::view::VisibleEntities};

use bevy_prototype_lyon::prelude::*;

use crate::{
    graph::{context::CurrentContext, nodes::ContextRoot, graph_cam::GraphCamera},
    events::nodes::*};

use self::{
    context_menu::{context_menu_button_system, spawn_context_menu}, 
    mode_menu::{create_mode_menu, mode_button_system, update_active_mode_highlight}, 
    nodes::NodesUiPlugin, edges::EdgeUiPlugin, 
    create_node_menu::CreateNodeMenuPlugin, grid::InfiniteGrid2DPlugin,
};

// Building blocks of specific components
mod modal;

mod context_menu;
mod mode_menu;
mod create_node_menu;
pub mod grid;
pub(crate) mod nodes;
pub(crate) mod edges;

pub struct KartaUiPlugin;

impl Plugin for KartaUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(ShapePlugin)

            // Resources
            .add_systems(PreStartup, default_font_setup)
            .add_systems(PreUpdate, 
                default_font_set.run_if(resource_exists::<FontHandle>()))

            .add_systems(Startup, gizmo_settings)

            .add_plugins(NodesUiPlugin)
            .add_plugins(EdgeUiPlugin)
            .add_plugins(CreateNodeMenuPlugin)
            .add_plugins(InfiniteGrid2DPlugin)
            
            // Element Systems
            .add_systems(Update, modal::modal_position_system.after(spawn_context_menu))

            // Systems
            .add_systems(Startup, create_mode_menu)
            .add_systems(Startup, create_context_and_active_bar)
            
            .add_systems(Update, update_context_label.run_if(resource_changed::<CurrentContext>()))
            
            .add_systems(Update, mode_button_system)
            .add_systems(Update, update_active_mode_highlight.after(mode_button_system))
            
            .add_systems(Update, context_menu_button_system)
            
            .add_systems(Update, context_menu::despawn_context_menus)
            .add_systems(
                Update, 
                spawn_context_menu.run_if(on_event::<NodeClickEvent>())
            )
            
            .add_systems(PostUpdate, point_to_root_if_offscreen)
        ;
    }
}

fn default_font_set(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    font_handle: Res<FontHandle>,
){
    if let Some(font) = fonts.remove(&font_handle.0) {
        fonts.add(font);
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
            focus_policy: FocusPolicy::Pass,
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

fn point_to_root_if_offscreen(
    mut gizmos: Gizmos,
    query: Query<(Entity, &GlobalTransform), With<ContextRoot>>,
    cameras: Query<(&GlobalTransform, &VisibleEntities), With<GraphCamera>>,
){
    for (campos, entities) in cameras.iter() {
        for (node, nodepos) in query.iter() {

            // Check if the entity is within the camera's view bounds
            if entities.entities.contains(&node)
            {
                continue;
            } else {
                // Find center of screen

                gizmos.line_2d(
                    campos.translation().truncate(), 
                    nodepos.translation().truncate(), 
                    Color::rgb(0.9, 0.9, 0.9),
                )
            }
        }
    }
}