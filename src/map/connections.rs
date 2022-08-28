use bevy::prelude::*;

use super::{ConnectionSide, LoadRoomConfig, MapManager};
use crate::{
    asset_loaders::{cache::AssetCache, EmbeddedAssets},
    collision::CollisionEvent,
    player::PlayerMovement,
};

#[derive(Component, Debug)]
pub struct Connection(pub(super) LoadRoomConfig, pub(super) ConnectionSide);

pub fn connection_collision_system(
    mut commands: Commands,
    mut collision_reader: EventReader<CollisionEvent>,
    connections_query: Query<&Connection>,
    mut player_query: Query<&mut Transform, With<PlayerMovement>>,
    mut asset_cache: ResMut<AssetCache<EmbeddedAssets>>,
    mut assets: ResMut<Assets<Image>>,
    mut map: ResMut<MapManager>,
) {
    for collision in collision_reader.iter() {
        if let Ok(connection) = connections_query.get(collision.static_entity) {
            if !connection.1.matches_collision(&collision.collision) {
                continue;
            }

            // dbg!(&collision);
            if let Ok(mut player_trans) = player_query.get_mut(collision.moving_entity) {
                if let Some(room) = map.room_stack.pop() {
                    commands.entity(room.entity).despawn_recursive();
                }
                let spawn_point = map
                    .load_room(
                        &mut asset_cache,
                        &mut assets,
                        &mut commands,
                        connection.0.clone(), // TODO this clone could be eliminated with more 'static
                        Some(connection.1.inverse()),
                    )
                    .unwrap_or_else(|e| panic!("Error loading room: {}", e));
                if let Some(spawn_point) = spawn_point {
                    player_trans.translation = spawn_point.spawn_point;
                }
            }
        }
    }
}
