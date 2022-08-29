// Allow very complex types for bevy queries
#![allow(clippy::type_complexity)]

//! Game submission for the second bevy jam
//! # Panics
//! When the initial room could not be loaded

mod asset_loaders;
mod camera;
mod checkpoint;
mod collision;
mod combat;
mod enemies;
mod map;
mod physics;
mod player;
mod util;

use bevy::prelude::*;

use bevy_framepace::{FramepacePlugin, FramepaceSettings, Limiter};

use asset_loaders::{cache::AssetCache, EmbeddedAssets};
use camera::FollowEntity;
use collision::CollisionEvent;
use enemies::EnemyPlugin;
use map::{connections, LoadRoomConfig, MapManager};
use physics::{PhysicsPlugin, VEL_MOVE_STAGE};
use player::{MouseCursor, PlayerPlugin};

const PLAYER_SIZE: f32 = 16.0;

/// Stage to move the camera in (TODO)
pub const CAMERA_MOVE_STAGE: &str = "cam_mov";
/// Stage run before `PostUpdate` (before transforms get propagated)
pub const LATE_UPDATE_STAGE: &str = "late_upd";

const COLLISION_STAGE: &str = "coll_stage";
const POST_COLLISION_STAGE: &str = "post_coll_stage";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_stage_before(
            CoreStage::PostUpdate,
            LATE_UPDATE_STAGE,
            SystemStage::parallel(),
        )
        .add_plugin(FramepacePlugin)
        .add_plugin(PhysicsPlugin)
        .add_stage_after(VEL_MOVE_STAGE, CAMERA_MOVE_STAGE, SystemStage::parallel())
        .add_stage_after(VEL_MOVE_STAGE, COLLISION_STAGE, SystemStage::parallel())
        .add_stage_after(
            COLLISION_STAGE,
            POST_COLLISION_STAGE,
            SystemStage::parallel(),
        )
        .add_plugin(PlayerPlugin)
        .add_plugin(EnemyPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(initial_room_setup)
        .add_startup_system(grab_mouse)
        .add_system(checkpoint::checkpoint_system)
        .add_system(combat::move_projectile_system)
        .add_system_to_stage(CAMERA_MOVE_STAGE, camera::camera_follow_system)
        .add_system_to_stage(COLLISION_STAGE, collision::collision_system)
        .add_system_to_stage(POST_COLLISION_STAGE, collision::collision_move_system)
        .add_system(connections::connection_collision_system)
        .add_event::<CollisionEvent>()
        .insert_resource(AssetCache::<EmbeddedAssets>::new())
        .insert_resource(MapManager::load_map("maps/main.toml", "demo".into()))
        .run();
}

/// Create the main game world
pub fn setup_system(mut commands: Commands, mut framepace: ResMut<FramepaceSettings>) {
    framepace.limiter = Limiter::from_framerate(27.0);

    commands
        .spawn_bundle(Camera2dBundle {
            projection: OrthographicProjection {
                scale: 0.2,
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 0.0, 10.0),
            ..Default::default()
        })
        .insert(FollowEntity);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(2.0, 2.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MouseCursor);
}

fn initial_room_setup(
    mut commands: Commands,
    mut asset_cache: ResMut<AssetCache<EmbeddedAssets>>,
    mut assets: ResMut<Assets<Image>>,
    mut map_manager: ResMut<MapManager>,
) {
    if let Err(e) = map_manager.load_room(
        &mut asset_cache,
        &mut assets,
        &mut commands,
        LoadRoomConfig {
            section: None,
            // room: "tt_need_earth".into(),
            // room: "s1_combine".into(),
            // room: "tt_fall_lr_checkpoint".into(),
            // room: "tt_need_fire_earth".into(),
            variation: None,
            room: "tt_get_earth".into(),
            // variation: Some(0),
        },
        None,
        None,
    ) {
        panic!("Could not load initial room: {}", e);
    }
}

fn grab_mouse(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}
