use std::ops::Index;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use std::string::FromUtf8Error;
use bevy::prelude::*;
use bevy::utils::HashMap;
use image::{ImageBuffer, Rgba};
use serde::de::DeserializeOwned;
use thiserror::Error;
use crate::asset_loaders::{AssetLoadError, EmbeddedAssetLoader, EmbeddedAssets, EmbeddedData};

#[derive(Deserialize)]
pub struct Map {
    sprites: HashMap<String, PathBuf>,
    sections: HashMap<String, Section>,
}

#[derive(Deserialize)]
pub struct Section {
    base_dir: PathBuf,
}

#[derive(Deserialize)]
pub struct Room {
    layers: u32,
    colors: HashMap<String, String>,
    connections: HashMap<String, String>,
}

pub fn map_as_resource(filename: &str) -> Map {
    match load_toml::<Map, &str>(filename) {
        Ok(map) => map,
        Err(e) => {
            panic!("There was an error parsing the map: {}", e);
        }
    }
}

#[derive(Error, Debug)]
pub enum LoadRoomError {
    #[error("The given section could not be found: {0}")]
    SectionNotFoundError(String),
    #[error("The given room's assets could not be loaded: {0}")]
    ParseError(AssetLoadError),
    #[error("The given room's config file could not be parsed: {0}")]
    RoomParseError(ParseError),
    #[error("Could not load layer: {0}")]
    LoadLayerError(LoadLayerError),
}

pub fn load_room_sprites(assets: &mut Assets<Image>, commands: &mut Commands, map: &Map, section_id: &str, room_id: &str) -> Result<(), LoadRoomError> {
    if let Some((_, sec)) = map.sections.iter().find(|(s, _)| *s == section_id) {
        let room: Room = load_toml(sec.base_dir.join(room_id).join("room.toml")).map_err(LoadRoomError::RoomParseError)?;
        for i in 0..room.layers {
            load_layer_file(
                assets,
                commands,
                map,
                &room,
                sec.base_dir.join(room_id).join(format!("layer{}.png", i)),
            ).map_err(LoadRoomError::LoadLayerError)?;
        }
        Ok(())
    } else {
        Err(LoadRoomError::SectionNotFoundError(section_id.to_owned()))
    }
}

#[derive(Error, Debug)]
pub enum LoadLayerError {
    #[error("Could not load layer file: {0}")]
    LoadError(AssetLoadError),
    #[error("The color in the image could not be found its corresponding config file: {0}")]
    InvalidColor(String),
    #[error("The color assigned sprite could not be found in the config file: {0}")]
    InvalidSprite(String),
}

fn load_layer_file<P: AsRef<Path>>(assets: &mut Assets<Image>, commands: &mut Commands, map: &Map, room: &Room, path: P) -> Result<(), LoadLayerError> {
    let image = EmbeddedData::load_image::<P, Rgba<u8>>(path)
        .map_err(LoadLayerError::LoadError)?;
    for (i, pixel) in image.pixels().enumerate() {
        let x = i % image.width() as usize;
        let y = i / image.width() as usize;
        if pixel.0[3] != 0 {
            let color_hex = format!("#{}", hex::encode(pixel.0.into_iter().take(3).collect::<Vec<u8>>()));
            let sprite_id = room.colors.get(&color_hex).ok_or_else(|| LoadLayerError::InvalidColor(color_hex))?;
            let sprite_path = map.sprites.get(sprite_id).ok_or_else(|| LoadLayerError::InvalidSprite(sprite_id.to_string()))?;
            commands.spawn_bundle(SpriteBundle {
                texture: EmbeddedAssets::load_image_as_asset(assets, sprite_path).map_err(LoadLayerError::LoadError)?,
                sprite: Sprite {
                    custom_size: Some(Vec2::new(128.0, 128.0)),
                    ..Default::default()
                },
                ..Default::default()
            })
                .insert(Transform {
                    translation: Vec3::new((x * 128) as f32, (y * 128) as f32, 0.0),
                    ..Default::default()
                });
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum ParseError {
    #[error("Failed to load file: {0}")]
    LoadError(AssetLoadError),
    #[error("Failed to parse file as UTF-8: {0}")]
    ParseFileError(FromUtf8Error),
    #[error("Failed to parse file as TOML: {0}")]
    ParseError(toml::de::Error),
}

fn load_toml<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, ParseError> {
    EmbeddedData::load(path)
        .map_err(ParseError::LoadError)
        .and_then(|data| String::from_utf8(data).map_err(ParseError::ParseFileError))
        .and_then(|s| toml::from_str(&s).map_err(ParseError::ParseError))
}
