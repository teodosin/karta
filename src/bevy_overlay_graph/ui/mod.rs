// All that is fixed in place in the foreground
// Excludes the graph and floating windows(?)

use bevy::{prelude::*, ui::{FocusPolicy, UiSystem}, render::view::VisibleEntities};

use bevy_mod_picking::backends::raycast::RaycastBackendSettings;
use bevy_prototype_lyon::prelude::*;

use crate::{
    graph::{context::CurrentContext, nodes::ContextRoot}, scene::scene::CurrentActive, vault::CurrentVault,
};

use self::{
    context_menu::{context_menu_button_system, spawn_node_context_menu, spawn_edge_context_menu}, 
    nodes::NodesUiPlugin, edges::EdgeUiPlugin, 
    grid::InfiniteGrid2DPlugin, graph_cam::GraphCamera, asset_manager::{ImageLoadTracker, on_image_load},
};

use super::events::{nodes::NodeClickEvent, edges::EdgeClickEvent};

// Building blocks of specific components
pub mod popup;

pub(crate) mod context_menu;
mod mode_menu;
pub(crate) mod grid;
pub(crate) mod ui_base_panel;
pub(crate) mod nodes;
pub(crate) mod edges;
pub(crate) mod graph_cam;
pub(crate) mod simulation;
pub(crate) mod asset_manager;

pub struct KartaUiPlugin;

impl Plugin for KartaUiPlugin {
    fn build(&self, app: &mut App) {
        app
            // Plugins
            .add_plugins(ShapePlugin)

            .add_plugins(graph_cam::GraphCamPlugin)
            .add_plugins(simulation::GraphSimPlugin)

            .add_plugins(ui_base_panel::UiNodePlugin)

            .insert_resource(ImageLoadTracker::new())

            // Resources
            .add_systems(PreStartup, require_markers_for_raycasting)
            // .add_systems(PreStartup, default_font_setup)
            .add_systems(PreStartup, 
                default_font_set.run_if(resource_exists::<FontHandle>()))

            .add_systems(Startup, gizmo_settings)

            .add_plugins(NodesUiPlugin)
            .add_plugins(EdgeUiPlugin)
            .add_plugins(InfiniteGrid2DPlugin)
            
            // Element Systems
            .add_systems(PostUpdate, popup::popup_position_system.after(UiSystem::Layout))

            // Systems

            // Modes are disabled until a workaround is decided for the way Bevy handles events since 0.12.1.
            // .add_systems(Startup, create_mode_menu)
            // .add_systems(Update, mode_button_system)
            // .add_systems(Update, update_active_mode_highlight.after(mode_button_system))

            .add_systems(PreUpdate, undo_redo_context)

            .add_systems(Startup, create_context_and_active_bar)
            
            .add_systems(Update, update_context_label.run_if(resource_changed::<CurrentContext>()))
            
            .add_systems(Update, context_menu_button_system)
            
            .add_systems(
                Update, 
                (
                    spawn_node_context_menu.run_if(on_event::<NodeClickEvent>()),
                    spawn_edge_context_menu.run_if(on_event::<EdgeClickEvent>())
                )
            )
            .add_systems(Update, context_menu::despawn_context_menus_on_any_click)
            
            .add_systems(PostUpdate, point_to_root_if_offscreen)
            
            .add_systems(PostUpdate, on_image_load)
        ;
    }
}

fn default_font_set(
    mut commands: Commands,
    mut fonts: ResMut<Assets<Font>>,
    asset_server: Res<AssetServer>,
    font_handle: Res<FontHandle>,
){
    if let Some(font) = fonts.remove(&font_handle.0) {
        fonts.add(font);
        commands.remove_resource::<FontHandle>();
    }

    let font = asset_server.load("fonts/Roboto/Roboto-Medium.ttf");
    commands.insert_resource(FontHandle(font));
}

fn default_font_setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,

) {
    let font = asset_server.load("fonts/Roboto/Roboto-Medium.ttf");
    commands.insert_resource(FontHandle(font));
}

fn require_markers_for_raycasting(
    mut settings: ResMut<RaycastBackendSettings>,
){
    settings.require_markers = true;
}

#[derive(Resource)]
struct FontHandle(Handle<Font>);

fn gizmo_settings(
    mut gizmo: ResMut<GizmoConfig>,
){
    gizmo.depth_bias = 1.0;
    gizmo.render_layers = bevy::render::view::RenderLayers::layer(31);
}


#[derive(Component)]
pub struct ContextLabel;

#[derive(Component)]
pub struct ActiveLabel;

fn undo_redo_context(
    vault: Res<CurrentVault>,
    mut context: ResMut<CurrentContext>,
    mut input: ResMut<Input<KeyCode>>,
){
    let vault_path = match &vault.vault {
        Some(vault) => vault.get_vault_path(),
        None => return,
    };

    if input.just_pressed(KeyCode::Q) {
        context.undo_context(&vault_path);
    }
    if input.just_pressed(KeyCode::E) {
        context.redo_context(&vault_path);
    }
}

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
    context: Res<CurrentContext>,
){
    match context.context {
        Some(ref cxt) => {
            for mut text in &mut query.iter_mut() {
                text.sections[0].value = cxt.get_path().to_string_lossy().to_string();
            }
        },
        None => {
            for mut text in &mut query.iter_mut() {
                text.sections[0].value = "No context".to_string();
            }
        }
    }
}

pub fn update_active_mode_label(
    mut query: Query<&mut Text, With<ActiveLabel>>,
    active: Res<CurrentActive>,
){
    match active.active {
        Some(ref active) => {
            for mut text in &mut query.iter_mut() {
                text.sections[0].value = active.to_string_lossy().to_string();
            }
        },
        None => {
            for mut text in &mut query.iter_mut() {
                text.sections[0].value = "None active".to_string();
            }
        }
    }
}

fn point_to_root_if_offscreen(
    mut gizmos: Gizmos,
    query: Query<(Entity, &GlobalTransform), With<ContextRoot>>,
    cameras: Query<(&GlobalTransform, &Camera, &VisibleEntities), With<GraphCamera>>,
){
    for (campos, cam, entities) in cameras.iter() {
        for (node, nodepos) in query.iter() {

            // Check if the entity is within the camera's view bounds
            if !entities.entities.contains(&node){
                gizmos.line_2d(
                    campos.translation().truncate(), 
                    nodepos.translation().truncate(), 
                    Color::rgb(0.9, 0.9, 0.9),
                );

                let viewport = match cam.viewport.as_ref(){
                    Some(v) => v,
                    None => continue,
                };

                println!("Viewport: {:?}", viewport);

                // Convert viewport size and position to Vec2 for calculations
                let viewport_size = Vec2::new(viewport.physical_size.x as f32, viewport.physical_size.y as f32);
                let viewport_position = Vec2::new(viewport.physical_position.x as f32, viewport.physical_position.y as f32);

                // Calculate camera and node positions in viewport space
                let campos_in_viewport = campos.translation().truncate() - viewport_position;
                let nodepos_in_viewport = nodepos.translation().truncate() - viewport_position;

                // Calculate intersection with each edge of the viewport
                let mut closest_point_in_viewport = Vec2::ZERO;
                let mut found = false;
                for edge in 0..4 {
                    let (p1, p2) = match edge {
                        0 => (Vec2::new(0.0, 0.0), Vec2::new(viewport_size.x, 0.0)),
                        1 => (Vec2::new(viewport_size.x, 0.0), Vec2::new(viewport_size.x, viewport_size.y)),
                        2 => (Vec2::new(viewport_size.x, viewport_size.y), Vec2::new(0.0, viewport_size.y)),
                        _ => (Vec2::new(0.0, viewport_size.y), Vec2::new(0.0, 0.0)),
                    };

                    if let Some(intersection) = line_intersection(campos_in_viewport, nodepos_in_viewport, p1, p2) {
                        closest_point_in_viewport = intersection;
                        found = true;
                        break;
                    }
                }

                if found {
                    // Convert the closest point in viewport space to world space
                    if let Some(closest_point_world) = cam.viewport_to_world_2d(campos, closest_point_in_viewport) {
                        gizmos.circle_2d(
                            closest_point_world,
                            500.0,
                            Color::rgb(0.9, 0.9, 0.9),
                        );
                    }
                }
            }
        }
    }
}

// Calculate line intersection
fn line_intersection(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2) -> Option<Vec2> {
    let s1 = p1 - p0;
    let s2 = p3 - p2;

    let s = (-s1.y * (p0.x - p2.x) + s1.x * (p0.y - p2.y)) / (-s2.x * s1.y + s1.x * s2.y);
    let t = (s2.x * (p0.y - p2.y) - s2.y * (p0.x - p2.x)) / (-s2.x * s1.y + s1.x * s2.y);

    if s >= 0.0 && s <= 1.0 && t >= 0.0 && t <= 1.0 {
        // Collision detected
        return Some(Vec2::new(p0.x + (t * s1.x), p0.y + (t * s1.y)));
    }

    None
}