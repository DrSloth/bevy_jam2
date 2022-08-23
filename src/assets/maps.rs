use serde::Deserialize;
use std::path::PathBuf;
use bevy::prelude::*;
use bevy::utils::HashMap;
use crate::assets::DataAssetServer;

#[derive(Deserialize)]
pub struct Map {
    sprites: HashMap<String, PathBuf>,
    sections: HashMap<String, Section>,
}

#[derive(Deserialize)]
pub struct Section {
    base_dir: PathBuf,
    room: Vec<String>,
    colors: HashMap<String, String>,
}

#[derive(thiserror::Error, Debug)]
pub enum MapParseError {
    #[error("failing parsing map: {0}")]
    ParseError(toml::de::Error),
}

// pub fn load_map_from_file(mut commands: Commands, server: Res<DataAssetServer>) -> Result<Map, MapParseError> {
//     toml::from_str("").map_err(MapParseError::ParseError)
// }

#[cfg(test)]
mod tests {

    async fn load_map_file() {

    }
}
