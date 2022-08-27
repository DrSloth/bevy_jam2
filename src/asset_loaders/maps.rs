use bevy::{prelude::*, utils::HashMap};
use std::path::{Path, PathBuf};

use image::Rgba;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::{
    asset_loaders::{AssetLoadError, EmbeddedAssetLoader, EmbeddedAssets, EmbeddedData},
    collision::{BreakableCollider, Collider, CollisionFilter},
    player::abilities::{collectibles::CollectibleAbilityTrigger, AbilityItem, ABILITY_MAP},
    AssetCache,
};

const TILE_SIZE: f32 = 8.0;

#[derive(Deserialize, Debug)]
pub struct Map {
    sprites: HashMap<String, TileConfig>,
    sections: HashMap<String, PathBuf>,
}

#[derive(Deserialize, Debug)]
pub struct TileConfig {
    sprite: Option<PathBuf>,
    #[serde(default)]
    zrot: i16,
    #[serde(default)]
    breakable: bool,
    item: Option<AbilityItem>,
    collision: Option<CollisionFilter>,
}

#[derive(Deserialize, Debug)]
pub struct Section {
    colors: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct Room {
    layers: Vec<String>,
    #[serde(default)]
    variations: Vec<Vec<String>>,
    connections: HashMap<String, NextRoom>,
    /// List of layers meant for collision
    #[serde(default)]
    collisions: HashMap<String, bool>,
}

#[derive(Deserialize, Debug)]
pub struct NextRoom {
    section_id: String,
    room_id: String,
    variation: Option<usize>,
}

pub fn load_map(filename: &str) -> Map {
    match load_toml(filename) {
        Ok(map) => map,
        Err(e) => {
            panic!("There was an error parsing map({}): {}", filename, e);
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

pub fn load_room(
    asset_cache: &mut AssetCache<EmbeddedAssets>,
    assets: &mut Assets<Image>,
    commands: &mut Commands,
    map: &Map,
    section_id: &str,
    room_id: &str,
    variation_id: Option<usize>,
) -> Result<(), LoadRoomError> {
    if let Some((_, section_path)) = map.sections.iter().find(|(s, _)| *s == section_id) {
        let section: Section =
            load_toml(section_path.join("section.toml")).map_err(LoadRoomError::RoomParseError)?;
        let room: Room = load_toml(section_path.join(room_id).join("room.toml"))
            .map_err(LoadRoomError::RoomParseError)?;

        let variation_iter = variation_id.iter().flat_map(|id| {
            room.variations
                .get(*id)
                .unwrap_or_else(|| panic!("Variation id {:?} does not exist", variation_id))
                .iter()
        });

        for (idx, layer) in (0i16..).zip(room.layers.iter().chain(variation_iter)) {
            if room.collisions.get(layer).map_or(false, |b| *b) {
                load_collision_layer(section_path.join(room_id).join(layer), commands)
                    .map_err(LoadRoomError::LoadLayerError)?;
            } else {
                load_layer(
                    asset_cache,
                    assets,
                    commands,
                    map,
                    &section.colors,
                    &room,
                    -idx,
                    section_path.join(room_id).join(layer),
                )
                .map_err(LoadRoomError::LoadLayerError)?;
            }
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

fn load_collision_layer<P: AsRef<Path>>(
    file_path: P,
    commands: &mut Commands,
) -> Result<(), LoadLayerError> {
    let mut colliders: HashMap<Rgba<u8>, (i16, i16)> = HashMap::new();

    let image =
        EmbeddedData::load_image::<Rgba<u8>, P>(file_path).map_err(LoadLayerError::LoadError)?;

    for (row, y) in image.rows().rev().zip(0i16..) {
        for (pixel, x) in row.zip(0i16..) {
            if pixel.0[3] != 0 {
                println!("Got pixel at ({}, {})", x, y);
                if let Some(pos) = colliders.remove(pixel) {
                    dbg!(pos, (x, y));
                    // Place collider
                    let width = f32::from(pos.0 - x) * TILE_SIZE;
                    let height = f32::from(y - pos.1) * TILE_SIZE;
                    let center = Vec2::new(
                        f32::from(x) * TILE_SIZE + width / 2.0,
                        f32::from(pos.1) * TILE_SIZE + height / 2.0,
                    );

                    let size = Vec2::new(width + TILE_SIZE, height + TILE_SIZE);
                    dbg!(width, height, center);
                    commands
                        .spawn_bundle(TransformBundle {
                            local: Transform::from_translation(center.extend(0.0)),
                            ..Default::default()
                        })
                        .insert(Collider { size, filter: CollisionFilter::ALL });

                    println!(
                        "Creating collider at {} with size {}",
                        center,
                        Vec2::new(width, height)
                    );
                } else {
                    colliders.insert(*pixel, (x, y));
                }
            }
        }
    }

    Ok(())
}

fn load_layer<P: AsRef<Path>>(
    asset_cache: &mut AssetCache<EmbeddedAssets>,
    assets: &mut Assets<Image>,
    commands: &mut Commands,
    map: &Map,
    colors: &HashMap<String, String>,
    room: &Room,
    z_index: i16,
    layer_path: P,
) -> Result<(), LoadLayerError> {
    let image =
        EmbeddedData::load_image::<Rgba<u8>, P>(layer_path).map_err(LoadLayerError::LoadError)?;
    for (row, y) in image.rows().rev().zip(0i16..) {
        for (pixel, x) in row.zip(0i16..) {
            if pixel.0[3] != 0 {
                let color_hex = format!("#{:02x}{:02x}{:02x}", pixel.0[0], pixel.0[1], pixel.0[2]);
                let sprite_id = colors
                    .get(&color_hex)
                    .ok_or_else(|| LoadLayerError::InvalidColor(color_hex.clone()))?;

                let tile_config = map
                    .sprites
                    .get(sprite_id)
                    .ok_or_else(|| LoadLayerError::InvalidSprite(sprite_id.clone()))?;

                let translation = Vec3::new(
                    f32::from(x) * TILE_SIZE,
                    f32::from(y) * TILE_SIZE,
                    f32::from(z_index),
                );
                let size = Vec2::splat(TILE_SIZE);

                if let Some(sprite_path) = &tile_config.sprite {
                    let texture = asset_cache
                        .load_image(assets, sprite_path)
                        .map_err(LoadLayerError::LoadError)?;

                    let mut tile = commands.spawn_bundle(SpriteBundle {
                        texture,
                        sprite: Sprite {
                            custom_size: Some(size),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation,
                            rotation: Quat::from_axis_angle(
                                Vec3::NEG_Z,
                                f32::from(tile_config.zrot).to_radians(),
                            ),
                            ..Default::default()
                        },
                        ..Default::default()
                    });

                    if let Some(filter) = tile_config.collision {
                        tile.insert(Collider { size, filter });
                    }

                    if tile_config.breakable {
                        tile.insert(BreakableCollider);
                    }

                    if let Some(item) = tile_config.item.and_then(|item| ABILITY_MAP.get(&item)) {
                        tile.insert(CollectibleAbilityTrigger::new_with_descriptor(
                            Vec2::new(32.0, 64.0),
                            Vec3::ZERO,
                            *item,
                        ));
                    }
                }

                if let Some(_next_room) = room.connections.get(&color_hex) {
                    commands.spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(size),
                            color: Color::rgb(0.0, 1.0, 0.0),
                            ..Default::default()
                        },
                        transform: Transform {
                            translation,
                            ..Default::default()
                        },
                        ..Default::default()
                    });
                }
            }
        }
    }
    Ok(())
}

#[derive(Error, Debug)]
pub enum TomlParseError {
    #[error("Failed to load file: {0}")]
    LoadError(AssetLoadError),
    #[error("Failed to parse file as TOML: {0}")]
    ParseError(toml::de::Error),
}

fn load_toml<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, TomlParseError> {
    EmbeddedData::load(path)
        .map_err(TomlParseError::LoadError)
        .and_then(|data| toml::from_slice(&data).map_err(TomlParseError::ParseError))
}
