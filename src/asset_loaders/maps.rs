use bevy::{prelude::*, utils::HashMap};
use std::{
    ops::Rem,
    path::{Path, PathBuf},
    string::FromUtf8Error,
};

use image::Rgba;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::{
    asset_loaders::{AssetLoadError, EmbeddedAssetLoader, EmbeddedAssets, EmbeddedData},
    collision::Collider,
};

const TILE_SIZE: f32 = 8.0;

#[derive(Deserialize, Debug)]
pub struct Map {
    sprites: HashMap<String, SpriteConfig>,
    sections: HashMap<String, Section>,
}

#[derive(Deserialize, Debug)]
pub struct SpriteConfig {
    path: PathBuf,
    #[serde(default)]
    zrot: i16,
}

#[derive(Deserialize, Debug)]
pub struct Section {
    base_dir: PathBuf,
}

#[derive(Deserialize, Debug)]
pub struct Room {
    layers: u32,
    colors: HashMap<String, String>,

    // TODO: Implement connections
    #[serde(rename = "connections")]
    _connections: HashMap<String, String>,
}

pub fn map_as_resource(filename: &str) -> Map {
    match load_toml(filename) {
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
    #[error("The given room's config file could not be parsed: {0}")]
    RoomParseError(TomlParseError),
    #[error("Could not load layer: {0}")]
    LoadLayerError(LoadLayerError),
}

pub fn load_room_sprites(
    assets: &mut Assets<Image>,
    commands: &mut Commands,
    map: &Map,
    section_id: &str,
    room_id: &str,
) -> Result<(), LoadRoomError> {
    if let Some((_, sec)) = map.sections.iter().find(|(s, _)| *s == section_id) {
        let room: Room = load_toml(sec.base_dir.join(room_id).join("room.toml"))
            .map_err(LoadRoomError::RoomParseError)?;
        for i in 0..room.layers {
            load_layer_file(
                assets,
                commands,
                map,
                &room,
                i,
                sec.base_dir.join(room_id).join(format!("layer{}.png", i)),
            )
            .map_err(LoadRoomError::LoadLayerError)?;
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

fn load_layer_file<P: AsRef<Path>>(
    assets: &mut Assets<Image>,
    commands: &mut Commands,
    map: &Map,
    room: &Room,
    layer_n: u32,
    path: P,
) -> Result<(), LoadLayerError> {
    let image = EmbeddedData::load_image::<P, Rgba<u8>>(path).map_err(LoadLayerError::LoadError)?;
    for (i, pixel) in image.pixels().enumerate() {
        let i: u32 = i
            .try_into()
            .unwrap_or_else(|e| panic!("Could not convert usize to u32: {}", e));
        let x = i.rem(image.width());
        let y = image
            .height()
            .saturating_sub(i.saturating_div(image.width()));
        if pixel.0[3] != 0 {
            let color_hex = format!("#{:x}{:x}{:x}", pixel.0[0], pixel.0[1], pixel.0[2]);
            let sprite_id = room
                .colors
                .get(&color_hex)
                .ok_or(LoadLayerError::InvalidColor(color_hex))?;

            let sprite_config = map
                .sprites
                .get(sprite_id)
                .ok_or_else(|| LoadLayerError::InvalidSprite(sprite_id.to_string()))?;

            let size = Vec2::splat(TILE_SIZE);
            #[allow(clippy::cast_precision_loss)]
            commands
                .spawn_bundle(SpriteBundle {
                    // TODO: Optimize: Reuse already loaded assets by saving handles
                    texture: EmbeddedAssets::load_image_as_asset(assets, &sprite_config.path)
                        .map_err(LoadLayerError::LoadError)?,
                    sprite: Sprite {
                        custom_size: Some(size),
                        ..Default::default()
                    },
                    transform: Transform {
                        translation: Vec3::new(
                            (x as f32) * TILE_SIZE,
                            (y as f32) * TILE_SIZE,
                            0.0 - (room.layers.saturating_sub(layer_n) as f32),
                        ),
                        rotation: Quat::from_axis_angle(Vec3::Z, f32::from(sprite_config.zrot)),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(Collider { size });
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum TomlParseError {
    #[error("Failed to load file: {0}")]
    LoadError(AssetLoadError),
    #[error("Failed to parse file as UTF-8: {0}")]
    ParseFileError(FromUtf8Error),
    #[error("Failed to parse file as TOML: {0}")]
    ParseError(toml::de::Error),
}

fn load_toml<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, TomlParseError> {
    EmbeddedData::load(path)
        .map_err(TomlParseError::LoadError)
        .and_then(|data| String::from_utf8(data).map_err(TomlParseError::ParseFileError))
        .and_then(|s| toml::from_str(&s).map_err(TomlParseError::ParseError))
}
