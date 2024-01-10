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
    render::color::Color, hierarchy::BuildChildren
};
use bevy_mod_picking::prelude::*;

pub struct UiNodePlugin;

impl Plugin for UiNodePlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, base_panel)
            .add_systems(PostUpdate, update_top_panel_colors)
        ;

    }
}

#[derive(Component)]
struct UiNode;

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
                border: UiRect {
                    left: Val::Px(2.0),
                    right: Val::Px(2.0),
                    top: Val::Px(0.0),
                    bottom: Val::Px(2.0),
                },
                ..Default::default()
            },
            background_color: BackgroundColor::from(Color::rgb(0.0, 0.0, 0.0)),
            border_color: BorderColor::from(col),
            ..default()
        },
        UiNode,
        Pickable::IGNORE,
        On::<Pointer<Drag>>::target_component_mut::<Style>(|drag, style| {
            style.left = Val::Px(drag.event.delta.x);
            style.top = Val::Px(drag.event.delta.y);

        }),
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