use bevy::{prelude::*, sprite::collide_aabb};
use serde::Deserialize;

use crate::{
    asset_loaders::{cache::AssetCache, EmbeddedAssets},
    collision::Collider,
    map::{ConnectionSide, LoadRoomConfig, MapManager, TILE_SIZE},
    player::{PlayerSpawn, PlayerTrigger},
};

#[derive(Debug, Component)]
pub struct CheckpointTrigger {
    pub size: Vec2,
    pub offset: Vec3,
    pub kind: CheckpointKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointKind {
    Return,
    Checkpoint,
    Reset,
}

pub fn checkpoint_system(
    mut commands: Commands,
    checkpoints_query: Query<(&CheckpointTrigger, &Transform), Without<PlayerTrigger>>,
    mut player_query: Query<(&mut Transform, &Collider, &mut PlayerTrigger)>,
    kb_input: ResMut<Input<KeyCode>>,
    mut map_manager: ResMut<MapManager>,
    mut asset_cache: ResMut<AssetCache<EmbeddedAssets>>,
    mut assets: ResMut<Assets<Image>>,
) {
    for (mut player_transform, player_collider, mut trigger) in player_query.iter_mut() {
        for (checkpoint, checkpoint_trans) in checkpoints_query.iter() {
            let collision = collide_aabb::collide(
                player_transform.translation,
                player_collider.size,
                checkpoint_trans.translation + checkpoint.offset,
                checkpoint.size,
            );

            if collision.is_some() {
                trigger.trigger_interact();
                if kb_input.just_pressed(KeyCode::W) {
                    match checkpoint.kind {
                        kind @ (CheckpointKind::Checkpoint | CheckpointKind::Reset) => {
                            match kind {
                                CheckpointKind::Reset => {
                                    if let Some(room) = map_manager.room_stack.pop() {
                                        commands.entity(room.entity).despawn_recursive();
                                    }
                                }
                                CheckpointKind::Checkpoint => {
                                    if let Some(room) = map_manager.room_stack.last_mut() {
                                        commands.entity(room.entity).despawn_recursive();
                                        room.player_position = Some(player_transform.translation);
                                    }
                                }
                                _ => unreachable!(),
                            }

                            map_manager
                                .load_room(
                                    &mut asset_cache,
                                    &mut assets,
                                    &mut commands,
                                    LoadRoomConfig {
                                        room: "checkpoint".into(),
                                        section: None,
                                        variation: None,
                                    },
                                    None,
                                    None,
                                )
                                .unwrap_or_else(|e| panic!("Unable to load checkpoint room {}", e));

                            player_transform.translation =
                                Vec3::new(15.0 * TILE_SIZE, 4.0 * TILE_SIZE, 0.0);
                        }
                        CheckpointKind::Return => {
                            println!("Retornar");
                            if let Some(room) = map_manager.room_stack.pop() {
                                commands.entity(room.entity).despawn_recursive();
                            }

                            let point = map_manager.load_previous_room(
                                &mut asset_cache,
                                &mut assets,
                                &mut commands,
                            );
                            player_transform.translation = point;
                        }
                    }
                }
            }
        }
    }
}
