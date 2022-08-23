use serde::Deserialize;
use std::path::PathBuf;
use std::string::FromUtf8Error;
use bevy::prelude::*;
use bevy::utils::HashMap;
use thiserror::Error;
use crate::asset_loaders::{AssetLoadError, EmbeddedAssetLoader, EmbeddedData};

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

#[derive(Error, Debug)]
pub enum MapParseError {
    #[error("Failed to load map file: {0}")]
    LoadError(AssetLoadError),
    #[error("Failed to parse file as UTF-8: {0}")]
    ParseFileError(FromUtf8Error),
    #[error("Failed parsing map: {0}")]
    ParseError(toml::de::Error),
}

pub fn insert_map_as_resource(commands: &mut Commands, filename: &str) -> Result<(), MapParseError> {
    load_map_from_file(filename).map(|map| commands.insert_resource(map))
}

fn load_map_from_file(filename: &str) -> Result<Map, MapParseError> {
    EmbeddedData::load(filename)
        .map_err(MapParseError::LoadError)
        .and_then(|data| String::from_utf8(data).map_err(MapParseError::ParseFileError))
        .and_then(|s| toml::from_str(&s).map_err(MapParseError::ParseError))
}
