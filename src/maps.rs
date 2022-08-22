use serde::Deserialize;
use std::path::Path;
use bevy::prelude::*;
use bevy::utils::HashMap;

#[derive(Deserialize)]
pub struct Map {
    sprites: HashMap<String, Path>,
    sections: HashMap<String, Section>,
}

#[derive(Deserialize)]
pub struct Section {
    base_dir: Path,
    room: Vec<String>,
    colors: HashMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum MapParseError {
    #[error("failing parsing map: {0}")]
    ParseError(toml::de::Error),
}

/// https://bevy-cheatbook.github.io/assets/assetserver.html
pub fn load_map_from_file(mut commands: Commands, server: Res<AssetServer>) -> Result<Map, MapParseError> {
    toml::from_str("").map_err(MapParseError::ParseError)
}

#[cfg(test)]
mod tests {

    async fn load_map_file() {

    }
}
