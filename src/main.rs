//! Game submission for the second bevy jam

mod assets;

use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_system)
        .add_system(player_move_system)
        .add_system(gravity_system)
        .run();
}

/// Create the main game world
pub fn setup_system(mut commands: Commands) {
    commands = assets::load_asset_servers(commands);

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
        .insert(Player)
        .insert(Gravity);
}

/// Component only added to the player character
#[derive(Component, Debug)]
pub struct Player;

/// System to move the player with input
pub fn player_move_system(
    mut player_query: Query<&mut Transform, With<Player>>,
    kb_input: ResMut<Input<KeyCode>>,
) {
    for mut player_transform in player_query.iter_mut() {
        for key in kb_input.get_pressed() {
            match key {
                KeyCode::A => player_transform.translation.x -= 1.0,
                KeyCode::D => player_transform.translation.x += 1.0,
                _ => (),
            }
        }
    }
}

/// Gravity component to make things fall
#[derive(Component, Debug)]
pub struct Gravity;

/// System to apply gravity to all entities with the Gravity components
pub fn gravity_system(mut query: Query<&mut Transform, With<Gravity>>) {
    const GRAVITY: f32 = 1.0;

    for mut transform in query.iter_mut() {
        transform.translation.y -= GRAVITY;
    }
}
