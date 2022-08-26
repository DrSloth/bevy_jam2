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

use bevy::{prelude::*, sprite::Mesh2dHandle};

use asset_loaders::maps;
use camera::FollowEntity;
use collision::{Collider, CollisionEvent, LineStart, Line};
use maps::Map;
use physics::{PhysicsPlugin, VEL_MOVE_STAGE};
use player::{
    abilities::{collectibles::CollectibleAbilityTrigger, PlayerDash, PlayerShoot},
    MouseCursor, PlayerPlugin,
};

const PLAYER_SIZE: f32 = 16.0;

/// Stage to move the camera in (TODO)
pub const CAMERA_MOVE_STAGE: &str = "cam_mov";
/// Stage run before `PostUpdate` (before transforms get propagated)
pub const LATE_UPDATE_STAGE: &str = "late_upd";

fn main() {
    App::new()
        .add_stage_before(
            CoreStage::PostUpdate,
            LATE_UPDATE_STAGE,
            SystemStage::parallel(),
        )
        .add_plugins(DefaultPlugins)
        .add_plugin(PhysicsPlugin)
        .add_stage_after(VEL_MOVE_STAGE, CAMERA_MOVE_STAGE, SystemStage::parallel())
        .add_plugin(PlayerPlugin)
        .add_startup_system(setup_system)
        .add_startup_system(grab_mouse)
        .add_system(combat::move_projectile_system)
        .add_system_to_stage(CAMERA_MOVE_STAGE, camera::camera_follow_system)
        .add_system(collision::collision_system)
        .add_event::<CollisionEvent>()
        .insert_resource(maps::map_as_resource("maps/main.toml"))
        .run();
}

/// Create the main game world
pub fn setup_system(
    mut commands: Commands,
    map: Res<Map>,
    mut assets: ResMut<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands.spawn_bundle(Camera2dBundle {
        projection: OrthographicProjection {
            scale: 0.2,
            ..Default::default()
        },
        transform: Transform::from_xyz(0.0, 0.0, 10.0),
        ..Default::default()
    });
    // .insert(FollowEntity);

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

    add_initial_room(&mut commands, &map, &mut assets);
    
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 1.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-13.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(LineStart);

    commands.spawn_bundle(ColorMesh2dBundle {
        mesh: Mesh2dHandle(meshes.add(Mesh::from(shape::Quad {
            size: Vec2::splat(1.5),
            ..Default::default()
        }))),
        transform: Transform::from_xyz(10.0,0.0,0.0),
        ..Default::default()
    }).insert(Line);

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(20.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Collider {
            size: Vec2::splat(4.0),
        });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, -20.0, 0.0),
            ..Default::default()
        })
        .insert(Collider {
            size: Vec2::splat(4.0),
        });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(-20.0, 0.0, 0.0),
            ..Default::default()
        })
        .insert(Collider {
            size: Vec2::splat(4.0),
        });

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(4.0, 4.0)),
                ..Default::default()
            },
            transform: Transform::from_xyz(0.0, 20.0, 0.0),
            ..Default::default()
        })
        .insert(Collider {
            size: Vec2::splat(4.0),
        });
}

fn add_initial_room(commands: &mut Commands, map: &Map, assets: &mut Assets<Image>) {
    if let Err(e) = maps::load_room_sprites(assets, commands, map, "tutorial", "room0", Some(0)) {
        panic!("Could not load initial room: {}", e);
    }
}

fn grab_mouse(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}
