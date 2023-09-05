//lib
use std::collections::HashMap;

use bevy::{input::mouse::*, prelude::*, sprite::MaterialMesh2dBundle};
use bevy_mod_picking::prelude::*;
use rand::Rng;

pub fn karta_app() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)

        .insert_resource(ViewSettings::default())

        .add_systems(Startup, setup)
        .add_systems(Startup, spawn_random_nodes)

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
struct Node;

#[derive(Component)]
struct Edge;

#[derive(Component)]
struct GraphPosition(Vec3);

impl Node {
    fn screamies(&self, num: &f32) {
        println!("{}", num);
    }
}
#[derive(Component)]
struct GraphColor(Color);

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle::default(),
        RaycastPickCamera::default()
    ));
}


fn move_node_selection(
    mut ev_mouse_drag: EventReader<MoveNodesEvent>,
    mut query: Query<(Entity, &Node, &mut Transform), With<Selected> >,
    mut click_evr: EventReader<MouseButtonInput>,
) {
    for ev in ev_mouse_drag.iter() {
        for (_entity, _node, mut transform) in query.iter_mut() {
            transform.translation.x += ev.value.x;
            transform.translation.y += ev.value.y;
        }
    }

    for ev in click_evr.iter() {
        if ev.button == MouseButton::Left  {
            for (_entity, _node, mut transform) in query.iter_mut() {
                transform.translation.x += 10.0;
                transform.translation.y += 10.0;
            }
        }
    }
}

#[derive(Event)]
struct MoveNodesEvent {
    value: Vec2,
}

fn spawn_random_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    for i in 0..10 {
        let mut rng = rand::thread_rng();

        commands.spawn((
            Node,
            MaterialMesh2dBundle {
                mesh: meshes.add(shape::Circle::new(50.).into()).into(),
                material: materials.add(ColorMaterial::from(Color::PURPLE)),
                transform: Transform::from_translation(Vec3::new(
                    rng.gen_range(-400.0..400.0),
                    rng.gen_range(-400.0..400.0),
                    i as f32,
                )),
                ..default()
            },
            PickableBundle::default(),
            RaycastPickTarget::default(),
            On::<Pointer<Drag>>::target_component_mut::<Transform>(|drag, transform| {
                transform.translation.x += drag.delta.x; // Make the square follow the mouse
                transform.translation.y -= drag.delta.y;
            }),
            //On::<Pointer<Drag>>::send_event::<MoveNodesEvent>(),
            On::<Pointer<Select>>::target_insert(Selected),
            On::<Pointer<Deselect>>::target_remove::<Selected>(),
        ));
    }
}

fn graph_pan(
    mut query: Query<(&mut Transform, &OrthographicProjection), With<Camera>>,
    windows: Query<&Window>,
    mut mot_events: EventReader<MouseMotion>,
    click_evr: Res<Input<MouseButton>>,
) {
    let window = windows.single();
    let cursor = window.cursor_position();

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
                view_settings.zoom = ev.y;
                for mut projection in query.iter_mut() {
                    let mut log_scale = projection.scale.ln();
                    log_scale -= ev.y * zoom_mult * time.delta_seconds();
                    projection.scale = log_scale.exp();
        
                    println!("Current zoom scale: {}", projection.scale);
            }},
            MouseScrollUnit::Pixel => (),
        }
    }
}

fn spread_nodes(mut query: Query<(Entity, &Node, &mut Transform)>) {
    let mut velocities: HashMap<Entity, Vec2> = HashMap::new();

    for (_entity, node, _transf) in query.iter() {
        node.screamies(&_transf.translation.x)
    }

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

    for (e, _src_node, mut transform) in query.iter_mut() {
        let _vel = velocities.get(&e).unwrap_or(&Vec2::ZERO);
        debug!("{:?}", "bozo".to_string());
        //transform.translation += Vec3::new(0.1, 0.1, 0.0);
    }
}
