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

use asset_loaders::maps;
use camera::FollowEntity;
use collision::CollisionEvent;
use maps::Map;
use physics::PhysicsPlugin;
use player::{
    abilities::{
        collectibles::CollectibleAbilityTrigger, PlayerDash, PlayerShoot,
    },
    MouseCursor, PlayerPlugin,
};

const PLAYER_SIZE: f32 = 16.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(grab_mouse)
        .add_system(combat::move_projectile_system)
        .add_system(camera::camera_follow_system)
                .add_system(collision::collision_system)
                .add_event::<CollisionEvent>()
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
            transform: Transform::from_xyz(58.0, 55.0, 0.0),
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

fn grab_mouse(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}
