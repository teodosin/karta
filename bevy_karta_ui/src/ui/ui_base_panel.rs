// Base entity for nodes pinned to the UI
// Should be extendable so that crate users can define their own types
// Will eventually be merged with GraphViewNodes once those are ready to be 
// transferred to using Bevy UI instead of prototype_lyon meshes

use bevy::{
    prelude::*,
    app::{
        Plugin, App, Startup
    }, 
    ecs::
        system::Commands
    , 
    ui::{
        node_bundles::{NodeBundle, ButtonBundle}, widget::Button
    }, 
    prelude::default, math::Vec2, 
    render::{color::Color, view::{self, RenderLayers}}, hierarchy::BuildChildren
};
use bevy_mod_picking::prelude::*;
use crate::ui::graph_cam::GraphCamera;

pub struct UiNodePlugin;

impl Plugin for UiNodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, base_panel)
            .add_systems(PostStartup, uinode_add_drag)
            .add_systems(PostUpdate, uinode_transform_to_style)
            // .add_systems(PostUpdate, update_top_panel_colors)
        ;

    }
}

#[derive(Component)]
struct UiNode(Vec2);

#[derive(Component)]
struct UiTopBar;

fn base_panel(
    mut commands: Commands,
){
    let position = Vec2::new(20.0, 20.0);
    let size = Vec2::new(100.0, 100.0);
    let col = Color::rgb(0.2, 0.2, 0.2);

    let base_panel = commands.spawn((
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                left: Val::Px(position.x),
                top: Val::Px(position.y),
                width: Val::Px(size.x),
                height: Val::Px(size.y),
                // border: UiRect {
                //     left: Val::Px(2.0),
                //     right: Val::Px(2.0),
                //     top: Val::Px(0.0),
                //     bottom: Val::Px(2.0),
                // },
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::rgb(0.0, 0.0, 0.0)),
            border_color: BorderColor::from(col),
            transform: Transform::from_xyz(position.x, position.y, 1000.0),
            ..default()
        },
        UiNode(position),
        Pickable::IGNORE,
    )).id();

    let top_bar = commands.spawn((
        ButtonBundle {
            style: Style {
                flex_direction: FlexDirection::Row,
                width: Val::Px(size.x),
                height: Val::Px(12.0),
                align_items: AlignItems::Start,
                align_self: AlignSelf::FlexEnd,
                justify_content: JustifyContent::Center,
                ..default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 1.0),
            background_color: BackgroundColor::from(col),
            ..default()
        },
        UiTopBar,
        PickableBundle::default(),
        // On::<Pointer<Drag>>::target_component_mut::<Style>(|drag, style| {
        //     style.left = Val::Px(drag.event.delta.x);
        //     style.top = Val::Px(drag.event.delta.y);

        // }),

    )).id();

    commands.entity(base_panel).push_children(&[top_bar]);
}

fn uinode_add_drag(
    mut commands: Commands,
    mut query: Query<Entity, Added<UiNode>>,
){
    for entity in &mut query.iter_mut() {
        commands.entity(entity).insert(
            On::<Pointer<Drag>>::listener_component_mut::<UiNode>(|drag, tform| {
                tform.0 += drag.event.delta;
            }),
        );
    }
}


fn uinode_transform_to_style(
    mut nodes: Query<(&UiNode, &mut Style), Changed<UiNode>>,
    window: Query<&Window>,
){
    if nodes.iter_mut().count() == 0 {
        return;
    }

    let viewport = window.single();

    let viewport = &viewport.resolution;
    let viewport = Vec2::new(viewport.physical_width() as f32, viewport.physical_height() as f32);

    for (transform, mut style) in &mut nodes.iter_mut() {
        let mut new_pos = Vec2::new(transform.0.x, transform.0.y);
        if new_pos.x < 0. {
            new_pos.x = 0.;
        }
        if new_pos.y < 0. {
            new_pos.y = 0.;
        }

        let width = style.width.resolve(1.0, Vec2::new(viewport.x as f32, viewport.y as f32));
        let width = width.unwrap();

        let height = style.height.resolve(1.0, Vec2::new(viewport.x as f32, viewport.y as f32));
        let height = height.unwrap();

        if new_pos.x > viewport.x as f32 - width {
            new_pos.x = viewport.x as f32 - width;
        }
        if new_pos.y > viewport.y as f32 - height {
            new_pos.y = viewport.y as f32 - height;
        }
        
        style.left = Val::Px(new_pos.x);
        style.top = Val::Px(new_pos.y);
    }
}


fn update_top_panel_colors(
    mut buttons: Query<(Option<&PickingInteraction>, &mut BackgroundColor), (With<Button>, With<UiTopBar>)>,
) {
    for (interaction, mut button_color) in &mut buttons {
        *button_color = match interaction {
            Some(PickingInteraction::Pressed) => Color::rgb(0.35, 0.75, 0.35),
            Some(PickingInteraction::Hovered) => Color::rgb(0.25, 0.25, 0.25),
            Some(PickingInteraction::None) | None => Color::rgb(0.15, 0.15, 0.15),
        }
        .into();
    }
}