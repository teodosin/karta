use axum::{extract::State, http::StatusCode, response::{IntoResponse, Json}};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::prelude::NodePath;

use super::AppState;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ColorTheme {
    pub viewport_bg: String,
    pub panel_bg: String,
    pub focal_hl: String,
    pub panel_hl: String,
    pub text_color: String,
    pub contrast_color: String,
    pub connection_color: String,
}

impl Default for ColorTheme {
    fn default() -> Self {
        ColorTheme {
            viewport_bg: "#000000".to_string(),
            panel_bg: "#431d1f".to_string(),
            focal_hl: "#f4902dff".to_string(),
            panel_hl: "#741f2fff".to_string(),
            text_color: "#f0f0f0".to_string(),
            contrast_color: "#60a5fa".to_string(),
            connection_color: "#505050".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct EdgeFilterSettings {
    pub contains_edges: String,
    pub normal_edges: String,
}

impl Default for EdgeFilterSettings {
    fn default() -> Self {
        EdgeFilterSettings {
            contains_edges: "always".to_string(),
            normal_edges: "always".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct KartaSettings {
    pub version: f32,
    pub save_last_viewed_context: bool,
    pub last_viewed_context_path: Option<NodePath>,
    pub vault_path: Option<String>,
    pub color_theme: ColorTheme,
    pub edge_filters: EdgeFilterSettings,
}

impl Default for KartaSettings {
    fn default() -> Self {
        KartaSettings {
            version: 0.1,
            save_last_viewed_context: true,
            last_viewed_context_path: None,
            vault_path: None,
            color_theme: ColorTheme::default(),
            edge_filters: EdgeFilterSettings::default(),
        }
    }
}

fn get_settings_path(vault_path: &str) -> PathBuf {
    let mut path = PathBuf::from(vault_path);
    path.push(".karta");
    path.push("settings.json");
    path
}

pub fn load_settings(vault_path: &str) -> KartaSettings {
    let settings_path = get_settings_path(vault_path);
    if settings_path.exists() {
        let content = fs::read_to_string(settings_path).unwrap_or_default();
        serde_json::from_str(&content).unwrap_or_else(|_| KartaSettings::default())
    } else {
        KartaSettings::default()
    }
}

pub fn save_settings(vault_path: &str, settings: &KartaSettings) -> Result<(), std::io::Error> {
    let settings_path = get_settings_path(vault_path);
    let parent_dir = settings_path.parent().unwrap();
    fs::create_dir_all(parent_dir)?;
    let content = serde_json::to_string_pretty(settings).unwrap();
    fs::write(settings_path, content)
}

pub async fn get_settings_handler(
    State(state): State<AppState>,
) -> Json<KartaSettings> {
    let service = state.service.read().unwrap();
    let vault_path = service.vault_fs_path().to_str().unwrap();
    Json(load_settings(vault_path))
}

pub async fn update_settings_handler(
    State(state): State<AppState>,
    Json(payload): Json<KartaSettings>,
) -> impl IntoResponse {
    let service = state.service.read().unwrap();
    let vault_path = service.vault_fs_path().to_str().unwrap();
    match save_settings(vault_path, &payload) {
        Ok(_) => (StatusCode::OK, Json(payload)).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Failed to save settings: {}", e),
        )
            .into_response(),
    }
}