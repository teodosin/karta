use bevy::prelude::{Resource, Plugin, App, Color};





pub struct ThemePlugin;

impl Plugin for ThemePlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(OutlinesTheme::default())
        ;
    }
}

// Hardcoded constants for now. Some UI functions are called directly
// and not as systems, so they can't access resources easily. This is
// cumbersome, so the resources won't be used until this has been 
// addressed. 

// Outlines
// ------------------------------------------------------------------
pub const OUTLINE_BASE_COLOR: Color = Color::rgba(0.2, 0.2, 0.4, 0.0);
pub const OUTLINE_HOVER_COLOR: Color = Color::rgb(0.3, 0.3, 0.5);
// pub const OUTLINE_SELECTED_COLOR: Color = Color::rgb(0.4, 0.4, 0.6);
// pub const OUTLINE_FOCAL_COLOR: Color = Color::ORANGE;
// pub const OUTLINE_FOCAL_HOVER_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);
// pub const OUTLINE_FOCAL_SELECTED_COLOR: Color = Color::rgb(0.9, 0.9, 0.9);

#[derive(Resource)]
pub struct OutlinesTheme {
    pub base_color: Color,
    pub hover_color: Color,
    pub focal_color: Color,
}

impl Default for OutlinesTheme {
    fn default() -> Self {
        OutlinesTheme {
            base_color: Color::rgb(0.2, 0.2, 0.4),
            hover_color: Color::rgb(0.9, 0.9, 0.9),
            focal_color: Color::ORANGE,
        }
    }
}

// Edges 
// ------------------------------------------------------------------
// parent edges are of a reddish hue. 
// base edges are of a bluish grey hue.
// focal edges are generally darker and more transparent

// Would color mixing be useful here?

// The previous 8 colors as consts
// pub const EDGE_FOCAL_BASE_COLOR: Color = Color::rgba(0.67, 0.21, 0.0, 0.5);
// pub const EDGE_FOCAL_BASE_HOVER_COLOR: Color = Color::rgba(0.73, 0.22, 0.0, 0.73);
// pub const EDGE_FOCAL_PARENT_COLOR: Color = Color::rgba(0.57, 0.0, 0.0, 0.5);
// pub const EDGE_FOCAL_PARENT_HOVER_COLOR: Color = Color::rgba(0.67, 0.0, 0.0, 0.73);
// pub const EDGE_BASE_COLOR: Color = Color::rgb(0.2, 0.2, 0.3);
// pub const EDGE_BASE_HOVER_COLOR: Color = Color::rgb(0.3, 0.3, 0.4);
pub const EDGE_PARENT_COLOR: Color = Color::rgb(0.67, 0.21, 0.0);
// pub const EDGE_PARENT_HOVER_COLOR: Color = Color::rgb(0.73, 0.22, 0.0);