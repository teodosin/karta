use std::collections::HashMap;

use bevy::{input::mouse::MouseWheel, prelude::*, sprite::MaterialMesh2dBundle};
use rand::Rng;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ViewSettings::default())
        .add_startup_system(setup)
        .add_startup_system(spawn_random_nodes)
        .add_event::<MouseScrollEvent>()
        .add_system(mouse_scroll_events)
        .add_system(graph_zoom)
        .add_system(spread_nodes)
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

#[derive(Component)]
struct Node;

#[derive(Component)]
struct Edge;

#[derive(Component)]
struct GraphPosition(Vec3);

#[derive(Component)]
struct GraphColor(Color);

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn create_window(mut commands: Commands) {
    commands.spawn(Window {
        title: "Karta".to_string(),
        ..default()
    });
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
        ));
    }
}

#[derive(Event)]
struct MouseScrollEvent {
    value: f32,
}

fn mouse_scroll_events(
    mut scroll_evr: EventReader<MouseWheel>,
    mut ev_mouse_scroll: EventWriter<MouseScrollEvent>,
    mut view_settings: ResMut<ViewSettings>,
) {
    use bevy::input::mouse::MouseScrollUnit;
    for ev in scroll_evr.iter() {
        match ev.unit {
            MouseScrollUnit::Line => {
                view_settings.zoom = ev.y;
                ev_mouse_scroll.send(MouseScrollEvent { value: ev.y });
            }
            MouseScrollUnit::Pixel => (),
        }
    }
}

fn graph_zoom(
    mut query: Query<&mut OrthographicProjection, With<Camera>>,
    time: Res<Time>,
    mut events: EventReader<MouseScrollEvent>,
) {
    let zoom_mult: f32 = 2.;

    for event in events.iter() {
        for mut projection in query.iter_mut() {
            let mut log_scale = projection.scale.ln();
            log_scale -= event.value * zoom_mult * time.delta_seconds();
            projection.scale = log_scale.exp();

            println!("Current zoom scale: {}", projection.scale);
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
                velocities.insert(esrc, vel.normalize() * 0.0000000000000000001);
                velocities.insert(etrgt, vel.normalize() * -0.0000000000000000001);
            }
        }
    }

    for (e, _src_node, mut transform) in query.iter_mut() {
        let vel = velocities.get(&e).unwrap_or(&Vec2::ZERO);
        transform.translation += Vec3::new(0.01, 0.01, 0.0);
    }
}
