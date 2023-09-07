//lib
use std::collections::HashMap;

use bevy::{input::mouse::*, prelude::*};
use bevy_mod_picking::prelude::*;

mod context;

use crate::context::ContextPlugin;

pub fn karta_app() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)

        .add_plugins(ContextPlugin)

        .insert_resource(ViewSettings::default())

        .add_systems(Startup, setup)

        .add_event::<MoveNodesEvent>()

        .add_systems(Update, move_node_selection)
        
        .add_systems(Update, graph_zoom)
        .add_systems(Update, graph_pan)

        .add_systems(Update, spread_nodes)

        .run();
}

#[derive(Resource, Debug)]
struct ViewSettings {
    pub zoom: f32,
}

impl Default for ViewSettings {
    fn default() -> Self {
        ViewSettings { zoom: 1.0 }
    }
}

#[derive(Resource, Default, Debug)]
struct InputSettings {}

#[derive(Component, Clone)]
struct Selected;

#[derive(Component)]
pub struct GraphNode;

#[derive(Component)]
struct Edge;

#[derive(Component)]
struct GraphPosition(Vec3);

impl GraphNode {
    fn screamies(&self, num: &f32) {
        println!("{}", num);
    }
}
#[derive(Component)]
struct GraphColor(Color);

fn setup(mut commands: Commands) {
    use bevy::core_pipeline::clear_color::ClearColorConfig;
    commands.spawn((
        Camera2dBundle {
            camera_2d: Camera2d {
                clear_color: ClearColorConfig::Custom(Color::rgb(0.1, 0.1, 0.1)),
            },
            ..Default::default()
        },
        RaycastPickCamera::default()
    ));
}


fn move_node_selection(
    mut ev_mouse_drag: EventReader<MoveNodesEvent>,
    mouse: Res<Input<MouseButton>>,
    view_settings: Res<ViewSettings>,
    mut query: Query<(Entity, &GraphNode, &mut Transform), With<Selected> >,
) {
    for ev in ev_mouse_drag.iter() {
        if mouse.pressed(MouseButton::Left){
            for (_entity, _node, mut transform) in query.iter_mut() {
                transform.translation.x += ev.delta.x * view_settings.zoom;
                transform.translation.y -= ev.delta.y * view_settings.zoom;
            }
        }
    }
}

#[derive(Event)]
struct MoveNodesEvent {
    delta: Vec2,
}

impl From<ListenerInput<Pointer<Drag>>> for MoveNodesEvent {
    fn from(event: ListenerInput<Pointer<Drag>>) -> Self {
        MoveNodesEvent {
            delta: event.delta, 
        }
    }
}

fn graph_pan(
    mut query: Query<(&mut Transform, &OrthographicProjection), With<Camera>>,
    windows: Query<&Window>,
    mut mot_events: EventReader<MouseMotion>,
    click_evr: Res<Input<MouseButton>>,
) {
    let window = windows.single();
    let _cursor = window.cursor_position();

    if click_evr.pressed(MouseButton::Middle) {
        for event in mot_events.iter() {
            for (mut transform, ortho) in query.iter_mut() {
                transform.translation.x -= event.delta.x * ortho.scale;
                transform.translation.y += event.delta.y * ortho.scale;
            }
        }
    }
}

fn graph_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    mut view_settings: ResMut<ViewSettings>,
    time: Res<Time>,
    mut events: EventReader<MouseWheel>,
) {
    let zoom_mult: f32 = 2.;

    for ev in events.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                for mut projection in query.iter_mut() {
                    let mut log_scale = projection.scale.ln();
                    log_scale -= ev.y * zoom_mult * time.delta_seconds();
                    projection.scale = log_scale.exp();
                    view_settings.zoom = projection.scale;
        
                    println!("Current zoom scale: {}", projection.scale);
            }},
            MouseScrollUnit::Pixel => (),
        }
    }
}

fn spread_nodes(mut query: Query<(Entity, &Node, &mut Transform)>) {
    let mut velocities: HashMap<Entity, Vec2> = HashMap::new();

    for (esrc, _src_node, transform) in query.iter() {
        let src_pos: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);
        let mut vel: Vec2;

        for (etrgt, _trgt_node, transform) in query.iter() {
            let trgt_pos: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);

            vel = trgt_pos - src_pos;

            if vel.length() < 100.0 {
                velocities.insert(esrc, vel.normalize() * 0.00001);
                velocities.insert(etrgt, vel.normalize() * -0.00001);
            }
        }
    }

    for (e, _src_node, mut _transform) in query.iter_mut() {
        let _vel = velocities.get(&e).unwrap_or(&Vec2::ZERO);
        debug!("{:?}", "bozo".to_string());
        //transform.translation += Vec3::new(0.1, 0.1, 0.0);
    }
}
