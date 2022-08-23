//! Game submission for the second bevy jam

mod collision;
mod cursor;
mod embed_io;
mod physics;
mod player;

use bevy::{asset::FileAssetIo, prelude::*};

use collision::{Collider, CollisionEvent, MovableCollider};
use embed_io::EmbedIo;
use physics::{Gravity, VelocityMap};
use player::{JumpEvent, PlayerMovement};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_system(player::player_input_system)
        .add_system(player::player_jump_system)
        .add_system(player::player_land_system)
        .add_system_to_stage(CoreStage::PostUpdate, player::player_fall_system)
        .add_system(physics::gravity_system)
        .add_system(physics::landing_system)
        .add_system(collision::collision_system)
        .add_system_to_stage(CoreStage::Last, physics::velocity_system)
        .add_event::<CollisionEvent>()
        .add_event::<JumpEvent>()
        .run();
}

/// Create the main game world
pub fn setup_system(mut commands: Commands) {
    commands.insert_resource(if cfg!(debug_assertions) {
        AssetServer::new(FileAssetIo::new("assets", true))
    } else {
        AssetServer::new(EmbedIo)
    });

    commands.spawn_bundle(Camera2dBundle::default());

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.0, 0.0),
                custom_size: Some(Vec2::new(50.0, 50.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(PlayerMovement::new())
        .insert(Gravity::new())
        .insert(VelocityMap::new())
        .insert(MovableCollider {
            size: Vec2::new(50.0, 50.0),
        });

    spawn_ground(
        &mut commands,
        Transform::from_xyz(0.0, -200.0, 0.0),
        Vec2::new(600.0, 35.0),
    );
    spawn_ground(
        &mut commands,
        Transform::from_xyz(0.0, 400.0, 0.0),
        Vec2::new(600.0, 35.0),
    );
    spawn_ground(
        &mut commands,
        Transform::from_xyz(300.0, 0.0, 0.0),
        Vec2::new(35.0, 600.0),
    );
    spawn_ground(
        &mut commands,
        Transform::from_xyz(-300.0, 0.0, 0.0),
        Vec2::new(35.0, 600.0),
    );

    commands.spawn_bundle(SpriteBundle {
        sprite: Sprite {
            color: Color::rgb(1.0, 0.5, 0.0),
            custom_size: Some(Vec2::new(20.0, 20.0)),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn spawn_ground(commands: &mut Commands, transform: Transform, size: Vec2) {
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(0.0, 0.0, 0.0),
                custom_size: Some(size),
                ..Default::default()
            },
            transform,
            ..Default::default()
        })
        .insert(Collider { size });
}
