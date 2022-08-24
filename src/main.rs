//! Game submission for the second bevy jam
//! # Panics
//! When the initial room could not be loaded

mod asset_loaders;
mod collision;
mod combat;
mod physics;
mod player;
mod util;

use bevy::prelude::*;
use crate::asset_loaders::maps;
use crate::maps::Map;

use collision::{Collider, CollisionEvent, MovableCollider};
use physics::{Gravity, VelocityMap};
use player::{
    abilities::{self, PlayerDash, PlayerInventory, PlayerShoot},
    JumpEvent, MouseCursor, PlayerMovement,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_startup_system(grab_mouse)
        .add_system(player::player_input_system)
        .add_system(player::player_jump_system)
        .add_system(player::player_land_system)
        .add_system(abilities::player_shoot_system)
        .add_system(combat::move_projectile_system)
        .add_system_to_stage(CoreStage::PreUpdate, player::move_cursor_system)
        .add_system_to_stage(CoreStage::PostUpdate, abilities::player_dash_system)
        .add_system_to_stage(CoreStage::PostUpdate, player::player_fall_system)
        .add_system(physics::gravity_system)
        .add_system(physics::landing_system)
        .add_system(collision::collision_system)
        .add_system_to_stage(CoreStage::Last, physics::velocity_system)
        .add_event::<CollisionEvent>()
        .add_event::<JumpEvent>()
        .insert_resource(maps::map_as_resource("maps/main.toml"))
        .run();
}

/// Create the main game world
pub fn setup_system(mut commands: Commands, map: Res<Map>, mut assets: ResMut<Assets<Image>>) {
    commands.spawn_bundle(Camera2dBundle::default());

    if let Err(e) = maps::load_room_sprites(&mut assets, &mut commands, &map, "tutorial", "room0") {
        panic!("Could not load initial room: {}", e);
    }

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
        .insert(PlayerShoot::default())
        .insert(Gravity::new())
        .insert(VelocityMap::new())
        .insert(PlayerDash::default())
        .insert(PlayerInventory::new_with::<PlayerShoot, PlayerDash>())
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
        Vec2::new(55.0, 600.0),
    );
    spawn_ground(
        &mut commands,
        Transform::from_xyz(-300.0, 0.0, 0.0),
        Vec2::new(55.0, 600.0),
    );

    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: Color::rgb(1.0, 0.5, 0.0),
                custom_size: Some(Vec2::new(20.0, 20.0)),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(MouseCursor);
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

fn grab_mouse(mut windows: ResMut<Windows>) {
    if let Some(window) = windows.get_primary_mut() {
        window.set_cursor_visibility(false);
        window.set_cursor_lock_mode(true);
    }
}
