pub mod connections;

use bevy::{prelude::*, sprite::collide_aabb::Collision, utils::HashMap};
use std::{
    borrow::Cow,
    path::{Path, PathBuf},
};

use image::Rgba;
use serde::{de::DeserializeOwned, Deserialize};
use thiserror::Error;

use crate::{
    asset_loaders::{AssetLoadError, EmbeddedAssetLoader, EmbeddedAssets, EmbeddedData},
    collision::{BreakableCollider, Collider, CollisionFilter},
    player::abilities::{collectibles::CollectibleAbilityTrigger, AbilityItem, ABILITY_MAP},
    AssetCache,
};
use connections::Connection;

pub type Colors = HashMap<String, String>;

pub const TILE_SIZE: f32 = 8.0;

// TODO all the ids should be interned, optimally with rust embed.
// TODO all the clones should be elided an replaced with either handles/or static references

/// Manager to manage loading rooms and parsing the map.
#[derive(Debug)]
pub struct MapManager {
    /// A stack of rooms to remember the last room(s)
    room_stack: Vec<Room>,
    map: Map,
    current_section: Section,
}

impl MapManager {
    pub fn load_map(filename: &str, section_name: Cow<'static, str>) -> Self {
        let map: Map = match load_toml(filename) {
            Ok(map) => map,
            Err(e) => {
                panic!("There was an error parsing map({}): {}", filename, e);
            }
        };

        let current_section = match map
            .sections
            .get(&*section_name)
            .ok_or_else(|| LoadMapError::SectionNotFoundError(section_name.clone()))
            .and_then(|section_path| {
                load_toml::<SectionConfig, _>(section_path.join("section.toml"))
            }) {
            Ok(section) => Section {
                colors: section.colors,
                name: section_name,
            },
            Err(e) => panic!("Failure to load section {}", e),
        };

        Self {
            room_stack: Vec::with_capacity(2),
            map,
            current_section,
        }
    }

    pub fn load_room(
        &mut self,
        asset_cache: &mut AssetCache<EmbeddedAssets>,
        assets: &mut Assets<Image>,
        commands: &mut Commands,
        load_room: LoadRoomConfig,
        spawn_direction: Option<ConnectionSide>,
    ) -> Result<Option<PlayerSpawnPoint>, LoadMapError> {
        let mut spawn_point: Option<Vec3> = None;
        // TODO reuse more of the buffers
        let section_path = if let Some(new_section) = load_room
            .section
            .and_then(|name| (name != self.current_section.name).then(|| name))
        {
            if let Some(section_path) = self.map.sections.get(&*new_section) {
                let section: SectionConfig = load_toml(section_path.join("section.toml"))?;
                self.current_section = Section {
                    name: new_section,
                    colors: section.colors,
                };

                section_path
            } else {
                return Err(LoadMapError::SectionNotFoundError(new_section));
            }
        } else {
            if let Some(section_path) = self.map.sections.get(&*self.current_section.name) {
                section_path
            } else {
                panic!("Map stores invalid section name");
            }
        };

        let room: RoomConfig = load_toml(section_path.join(&*load_room.room).join("room.toml"))?;

        let variation_iter = load_room.variation.iter().flat_map(|id| {
            room.variations
                .get(*id)
                .unwrap_or_else(|| panic!("Variation id {:?} does not exist", load_room.variation))
                .iter()
        });

        let room_parent = commands.spawn_bundle(SpatialBundle::default()).id();

        for (idx, layer) in (0i16..).zip(room.layers.iter().chain(variation_iter)) {
            if room.collisions.get(layer).map_or(false, |b| *b) {
                load_collision_layer(
                    commands,
                    room_parent,
                    section_path.join(&*load_room.room).join(layer),
                )?;
            } else {
                spawn_point = load_layer(
                    asset_cache,
                    assets,
                    commands,
                    Layer {
                        map: &self.map,
                        colors: &self.current_section.colors,
                        room: &room,
                        z_index: idx.wrapping_neg(),
                    },
                    room_parent,
                    spawn_direction,
                    section_path.join(&*load_room.room).join(layer),
                )?;
            }
        }

        let res = if let Some(spawn_dir) = spawn_direction {
            if let Some(spawn_point) = spawn_point {
                Ok(Some(PlayerSpawnPoint {
                    spawn_dir,
                    spawn_point,
                }))
            } else {
                panic!(
                    "Unconnected room {} with dir {:?}",
                    load_room.room, spawn_dir
                )
            }
        } else {
            Ok(None)
        };

        self.room_stack.push(Room {
            id: load_room.room,
            entity: room_parent,
        });

        res
    }
}

#[derive(Debug)]
pub struct Room {
    id: Cow<'static, str>,
    entity: Entity,
}

#[derive(Debug)]
pub struct Section {
    name: Cow<'static, str>,
    colors: Colors,
}

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
    #[serde(default)]
    connection: Option<ConnectionSide>,
}

#[derive(Deserialize, Debug)]
pub struct SectionConfig {
    colors: Colors,
}

#[derive(Deserialize, Debug)]
pub struct RoomConfig {
    layers: Vec<String>,
    #[serde(default)]
    variations: Vec<Vec<String>>,
    connections: HashMap<String, LoadRoomConfig>,
    /// List of layers meant for collision
    #[serde(default)]
    collisions: HashMap<String, bool>,
}

#[derive(Deserialize, Debug, Clone)]
pub struct LoadRoomConfig {
    pub section: Option<Cow<'static, str>>,
    pub room: Cow<'static, str>,
    pub variation: Option<usize>,
}

#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConnectionSide {
    Top,
    Right,
    Bottom,
    Left,
}

impl ConnectionSide {
    pub fn matches_collision(self, coll_dir: &Collision) -> bool {
        match (self, coll_dir) {
            (Self::Bottom, Collision::Top) => true,
            (Self::Top, Collision::Bottom) => true,
            (Self::Left, Collision::Right) => true,
            (Self::Right, Collision::Left) => true,
            _ => false,
        }
    }

    pub fn inverse(self) -> Self {
        match self {
            Self::Bottom => Self::Top,
            Self::Top => Self::Bottom,
            Self::Right => Self::Left,
            Self::Left => Self::Right,
        }
    }
}

fn load_collision_layer<P: AsRef<Path>>(
    commands: &mut Commands,
    parent: Entity,
    file_path: P,
) -> Result<(), LoadMapError> {
    let mut colliders: HashMap<Rgba<u8>, (i16, i16)> = HashMap::new();

    let image = EmbeddedData::load_image::<Rgba<u8>, P>(file_path)?;

    for (row, y) in image.rows().rev().zip(0i16..) {
        for (pixel, x) in row.zip(0i16..) {
            if pixel.0[3] != 0 {
                if let Some(pos) = colliders.remove(pixel) {
                    // Place collider
                    let width = f32::from(pos.0 - x) * TILE_SIZE;
                    let height = f32::from(y - pos.1) * TILE_SIZE;
                    let center = Vec2::new(
                        f32::from(x) * TILE_SIZE + width / 2.0,
                        f32::from(pos.1) * TILE_SIZE + height / 2.0,
                    );

                    let size = Vec2::new(width + TILE_SIZE, height + TILE_SIZE);
                    let ent = commands
                        .spawn_bundle(TransformBundle {
                            local: Transform::from_translation(center.extend(0.0)),
                            ..Default::default()
                        })
                        .insert(Collider {
                            size,
                            filter: CollisionFilter::ALL,
                        }).id();
                    
                    commands.entity(parent).push_children(&[ent]);
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
    layer: Layer,
    parent: Entity,
    spawn_dir: Option<ConnectionSide>,
    layer_path: P,
) -> Result<Option<Vec3>, LoadMapError> {
    let mut spawn_point: Option<Vec3> = None;
    let image = EmbeddedData::load_image::<Rgba<u8>, _>(layer_path.as_ref())?;
    for (row, y) in image.rows().rev().zip(0i16..) {
        for (pixel, x) in row.zip(0i16..) {
            if pixel.0[3] != 0 {
                let color_hex = format!("#{:02x}{:02x}{:02x}", pixel.0[0], pixel.0[1], pixel.0[2]);
                let sprite_id = layer
                    .colors
                    .get(&color_hex)
                    .ok_or_else(|| LoadMapError::InvalidColor(color_hex.clone()))?;

                let tile_config = layer
                    .map
                    .sprites
                    .get(sprite_id)
                    .ok_or_else(|| LoadMapError::InvalidSprite(sprite_id.clone()))?;

                let mut tile = commands.spawn();

                let translation = {
                    let translation = Vec3::new(
                        f32::from(x) * TILE_SIZE,
                        f32::from(y) * TILE_SIZE,
                        f32::from(layer.z_index),
                    );

                    if let Some(connection_side) = tile_config.connection {
                        if let Some(connection_config) = layer.room.connections.get(&color_hex) {
                            println!("{:?}", connection_side);
                            tile.insert(Connection(connection_config.clone(), connection_side))
                                .insert(Collider {
                                    size: Vec2::splat(TILE_SIZE),
                                    filter: CollisionFilter::ALL,
                                });

                            //TODO
                            let offset = match connection_side {
                                ConnectionSide::Bottom => Vec3::new(0.0, -TILE_SIZE, 0.0),
                                ConnectionSide::Top => Vec3::new(0.0, TILE_SIZE, 0.0),
                                ConnectionSide::Right => Vec3::new(TILE_SIZE, 0.0, 0.0),
                                ConnectionSide::Left => Vec3::new(-TILE_SIZE, 0.0, 0.0),
                            };

                            spawn_point = match (spawn_point, spawn_dir, connection_side) {
                                (None, Some(dir0), dir1) if dir0 == dir1 => {
                                    Some(translation.truncate().extend(0.0) - offset)
                                }
                                (spawn_point, _, _) => spawn_point,
                            };

                            translation + offset
                        } else {
                            panic!(
                                "Unset connection {} for {:?}",
                                color_hex,
                                layer_path.as_ref()
                            )
                        }
                    } else {
                        translation
                    }
                };

                let size = Vec2::splat(TILE_SIZE);

                let transform = Transform {
                    translation,
                    rotation: Quat::from_axis_angle(
                        Vec3::NEG_Z,
                        f32::from(tile_config.zrot).to_radians(),
                    ),
                    ..Default::default()
                };

                if let Some(sprite_path) = &tile_config.sprite {
                    let texture = asset_cache
                        .load_image(assets, sprite_path)
                        .map_err(LoadMapError::LoadError)?;
                    tile.insert_bundle(SpriteBundle {
                        texture,
                        sprite: Sprite {
                            custom_size: Some(size),
                            ..Default::default()
                        },
                        transform,
                        ..Default::default()
                    })
                } else {
                    tile.insert_bundle(TransformBundle {
                        local: transform,
                        ..Default::default()
                    })
                };

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

                let tile_id = tile.id();
                commands.entity(parent).push_children(&[tile_id]);
            }
        }
    }
    Ok(spawn_point)
}

pub struct PlayerSpawnPoint {
    pub spawn_dir: ConnectionSide,
    pub spawn_point: Vec3,
}

struct Layer<'room, 'map, 'colors> {
    room: &'room RoomConfig,
    map: &'map Map,
    colors: &'colors Colors,
    z_index: i16,
}

fn load_toml<T: DeserializeOwned, P: AsRef<Path>>(path: P) -> Result<T, LoadMapError> {
    let data = EmbeddedData::load(path)?;
    toml::from_slice(&data).map_err(Into::into)
}

#[derive(Debug, Error)]
pub enum LoadMapError {
    #[error("Error to load asset: {0}")]
    LoadError(#[from] AssetLoadError),
    #[error("Error to parse toml file {0}")]
    TomlParseError(#[from] toml::de::Error),
    #[error("The color in the image could not be found its corresponding config file: {0}")]
    InvalidColor(String),
    #[error("The color assigned sprite could not be found in the config file: {0}")]
    InvalidSprite(String),
    #[error("The given section could not be found: {0}")]
    SectionNotFoundError(Cow<'static, str>),
}
