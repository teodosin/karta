use std::collections::HashMap;

use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
use bevy_egui::{egui, EguiPlugin, EguiContexts};
use bevy_mod_picking::{PickableBundle, prelude::{RaycastPickCamera, RaycastPickTarget, OnPointer, Drag}, DefaultPickingPlugins};
use rand::Rng;


fn main() {

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(DefaultPickingPlugins)
        .add_plugin(EguiPlugin)

        .insert_resource(CurrentPath("/".to_string()))

        .add_startup_system(setup)
        .add_startup_system(spawn_random_nodes)
        
        .add_system(spread_nodes)
    
        .run();
    
}

#[derive(Resource, Default, Debug)]
struct CurrentPath(String);

#[derive(Component)]
struct Node;

#[derive(Component)]
struct Edge;

#[derive(Component)]
struct GraphPosition(Vec3);

#[derive(Component)]
struct GraphColor(Color);

fn setup(
    mut commands: Commands,
) {
    commands.spawn((
        Camera2dBundle::default(),
        RaycastPickCamera::default()
    ));
}

fn create_window(mut commands: Commands){
    commands.spawn(Window {
        title: "Karta".to_string(),
        ..default()
    });
}

fn spawn_random_nodes(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
){

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
                    i as f32)),
                ..default()
            },
            PickableBundle::default(),
            RaycastPickTarget::default(),
            OnPointer::<Drag>::target_component_mut::<Transform>(|drag, transform| {
                transform.translation += drag.delta.extend(0.0)
            })
        ));
    }

}

fn spread_nodes(
    mut query: Query<(Entity, &Node, &mut Transform)>,
) {
    let mut velocities: HashMap<Entity, Vec2> = HashMap::new();

    for (esrc, _src_node, transform) in query.iter(){
        let src_pos: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);
        let mut vel: Vec2;

        for (etrgt, _trgt_node, transform) in query.iter(){
            let trgt_pos: Vec2 = Vec2::new(transform.translation.x, transform.translation.y);

            vel = trgt_pos - src_pos;

            if vel.length() < 100.0 {
                velocities.insert(esrc, vel.normalize() * 0.0000000000000000001);
                velocities.insert(etrgt, vel.normalize() * -0.0000000000000000001);
            }
        };
    };

    for (e, _src_node, mut transform) in query.iter_mut(){
        let vel = velocities.get(&e).unwrap_or(&Vec2::ZERO);
        transform.translation += Vec3::new(0.01, 0.01, 0.0);
    };
    
}
