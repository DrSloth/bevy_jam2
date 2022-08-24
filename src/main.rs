// Allow very complex types for bevy queries
#![allow(clippy::type_complexity)]

//! Game submission for the second bevy jam
//! # Panics
//! When the initial room could not be loaded

mod asset_loaders;
mod camera;
mod collision;
mod combat;
mod physics;
mod player;
mod util;

use bevy::prelude::*;

use asset_loaders::{maps, EmbeddedAssetLoader, EmbeddedAssets};
use camera::{FollowEntity, FollowedByCamera};
use collision::{CollisionEvent, MoveableCollider};
use maps::Map;
use physics::{Gravity, VelocityMap};
use player::{
    abilities::{
        self,
        collectibles::{self, CollectibleAbilityTrigger},
        PlayerDash, PlayerInventory, PlayerShoot,
    },
    JumpEvent, MouseCursor, PlayerMovement,
};

const PLAYER_SIZE: f32 = 16.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_startup_system(grab_mouse)
        .add_system(camera::camera_follow_system)
        .add_system(player::player_input_system)
        .add_system(player::player_jump_system)
        .add_system(player::player_land_system)
        .add_system(abilities::player_shoot_system)
        .add_system(combat::move_projectile_system)
        .add_system(collectibles::collect_ability_system)
        .add_system_to_stage(CoreStage::PreUpdate, player::move_cursor_system)
        .add_system_to_stage(CoreStage::PostUpdate, abilities::player_dash_system)
        .add_system_to_stage(CoreStage::PostUpdate, player::player_fall_system)
        .add_system(physics::gravity_system)
        .add_system_to_stage(CoreStage::PostUpdate, physics::landing_system)
        .add_system(collision::collision_system)
        .add_system_to_stage(CoreStage::Last, physics::velocity_system)
        .add_event::<CollisionEvent>()
        .add_event::<JumpEvent>()
        .insert_resource(maps::map_as_resource("maps/main.toml"))
        .run();
}

/// Create the main game world
pub fn setup_system(mut commands: Commands, map: Res<Map>, mut assets: ResMut<Assets<Image>>) {
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

    add_initial_room(&mut commands, &map, &mut assets);
    add_player(&mut commands, &mut assets);

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

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.2, 1.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(38.0, 55.0, 0.0),
            ..Default::default()
        })
        .insert(CollectibleAbilityTrigger::new::<PlayerShoot>(
            Vec2::new(40.0, 600.0),
            Vec3::new(0.0, 0.0, 0.0),
        ));

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.2, 0.2, 1.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(5.0, 55.0, 0.0),
            ..Default::default()
        })
        .insert(CollectibleAbilityTrigger::new::<PlayerDash>(
            Vec2::new(40.0, 600.0),
            Vec3::new(0.0, 0.0, 0.0),
        ));
}

fn add_initial_room(commands: &mut Commands, map: &Map, assets: &mut Assets<Image>) {
    if let Err(e) = maps::load_room_sprites(assets, commands, map, "tutorial", "room0") {
        panic!("Could not load initial room: {}", e);
    }
}

fn add_player(commands: &mut Commands, assets: &mut Assets<Image>) {
    let texture =
        EmbeddedAssets::load_image_as_asset(assets, "sprites/character/movement/idle.png")
            .unwrap_or_else(|e| panic!("The player sprite could not be loaded: {}", e));
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::splat(PLAYER_SIZE)),
                ..Default::default()
            },
            texture,
            transform: Transform {
                translation: Vec3::new(1.0 * PLAYER_SIZE, 4.0 * PLAYER_SIZE, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FollowedByCamera)
        .insert(PlayerMovement::new())
        // .insert(PlayerShoot::default())
        .insert(Gravity::new())
        .insert(VelocityMap::new())
        // .insert(PlayerDash::default())
        .insert(PlayerInventory::new())
        // .insert(PlayerInventory::new_with::<PlayerShoot, PlayerDash>())
        .insert(MoveableCollider {
            size: Vec2::splat(PLAYER_SIZE),
        });
}

fn grab_mouse(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}
