use bevy::{app::{Plugin, App}, asset::{embedded_asset, AssetApp}};

use crate::prelude::grid::GridMaterial;

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        embedded_asset!(app, "grid_material.wgsl");
        app
            .init_asset::<GridMaterial>()
        ;
    }
}