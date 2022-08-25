pub mod abilities;

use bevy::{prelude::*, sprite::collide_aabb::Collision};

use self::abilities::PlayerInventory;
use crate::{
    asset_loaders::{EmbeddedAssetLoader, EmbeddedAssets},
    camera::FollowedByCamera,
    collision::{CollisionEvent, MoveableCollider},
    physics::{Gravity, VelocityId, VelocityMap, GRAVITY, VEL_SYSTEM_STAGE},
    PLAYER_SIZE,
};
use abilities::collectibles;

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(player_setup_system)
            .add_system(player_input_system)
            .add_system(player_jump_system)
            .add_system(player_land_system)
            .add_system(abilities::player_shoot_system)
            .add_system(collectibles::collect_ability_system)
            .add_system_to_stage(CoreStage::PreUpdate, move_cursor_system)
            .add_system_to_stage(CoreStage::PostUpdate, abilities::player_dash_system)
            .add_system_to_stage(CoreStage::PostUpdate, player_fall_system)
            .add_system_to_stage(VEL_SYSTEM_STAGE, add_player_velocity_system)
            .add_event::<JumpEvent>();
    }
}

fn player_setup_system(mut commands: Commands, mut assets: ResMut<Assets<Image>>) {
    let texture =
        EmbeddedAssets::load_image_as_asset(&mut assets, "sprites/character/movement/idle.png")
            .unwrap_or_else(|e| panic!("The player sprite could not be loaded: {}", e));

    let mut vel_map = VelocityMap::new();
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
        .insert(PlayerMovement::new_in(&mut vel_map))
        .insert(Gravity::new_in(&mut vel_map))
        .insert(vel_map)
        .insert(PlayerInventory::new())
        .insert(MoveableCollider {
            size: Vec2::splat(PLAYER_SIZE),
        });
}

#[derive(Debug)]
pub struct JumpEvent(pub Entity);

/// Component only added to the player character
#[derive(Component, Debug)]
pub struct PlayerMovement {
    vel_id: VelocityId,
    //TODO maybe use a state machine
    can_jump: bool,
    can_move: bool,
    pub velocity: Vec2,
}

impl PlayerMovement {
    /// Create a new player movement with the given `vel_id`
    pub fn new(vel_id: VelocityId) -> Self {
        Self {
            vel_id,
            can_jump: true,
            can_move: true,
            velocity: Vec2::ZERO,
        }
    }

    /// Create a new player movement inside the given [`VelocityMap`]
    pub fn new_in(vel_map: &mut VelocityMap) -> Self {
        Self::new(vel_map.register().0)
    }
}

/// System to move the player with input
pub fn player_input_system(
    mut player_query: Query<(&mut PlayerMovement, Entity)>,
    mut jump_event_writer: EventWriter<JumpEvent>,
    kb_input: ResMut<Input<KeyCode>>,
) {
    const SPEED: f32 = 2.0;

    for (mut player, entity) in player_query.iter_mut() {
        if !player.can_move {
            continue;
        }

        player.velocity.x = 0.0;
        for key in kb_input.get_pressed() {
            match key {
                KeyCode::A => player.velocity.x += -SPEED,
                KeyCode::D => player.velocity.x += SPEED,
                KeyCode::Space => jump_event_writer.send(JumpEvent(entity)),
                _ => (),
            }
        }
    }
}

pub fn player_jump_system(
    mut player_query: Query<(&VelocityMap, &mut PlayerMovement, &Gravity)>,
    mut jump_event_reader: EventReader<JumpEvent>,
) {
    const JUMP_POWER: f32 = 7.5;

    for JumpEvent(entity) in jump_event_reader.iter() {
        if let Ok((vel_map, mut player_movement, grav)) = player_query.get_mut(*entity) {
            let falling = grav_is_falling(grav, vel_map);
            let can_jump = !falling && player_movement.can_jump;

            if can_jump {
                player_movement.velocity.y = JUMP_POWER;
                player_movement.can_jump = false;
            }
        }
    }
}

pub fn player_land_system(
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut player_query: Query<(&mut PlayerMovement, &Gravity, &VelocityMap)>,
) {
    for collision in collision_event_reader.iter() {
        if let Collision::Top = collision.collision {
            if let Ok((mut player, grav, vel_map)) = player_query.get_mut(collision.entity) {
                let player_y_speed = player.velocity.y.abs();

                if let Some(grav_vel) = vel_map.get(grav.vel_id) {
                    if !is_falling(grav_vel.y) && player_y_speed < GRAVITY && !player.can_jump {
                        player.can_jump = true;
                    }
                }
            }
        }
    }
}

fn is_falling(grav_y_vel: f32) -> bool {
    grav_y_vel < -(GRAVITY * 3.0)
}

fn grav_is_falling(grav: &Gravity, vel_map: &VelocityMap) -> bool {
    vel_map
        .get(grav.vel_id)
        .map_or(true, |vel| is_falling(vel.y))
}

/// Makes the player slow down while falling
pub fn player_fall_system(mut player_query: Query<(&mut PlayerMovement, &mut Gravity)>) {
    const PLAYER_FALL_MULTIPLIER: f32 = 1.2;

    for (mut player, mut gravity) in player_query.iter_mut() {
        if player.velocity.y > 0.0 {
            player.velocity.y += gravity.velocity.y;
            gravity.velocity = Vec2::ZERO;
        } else if gravity.velocity.y < -(GRAVITY * 2.0) {
            // TODO maybe this should be a gravity scale in the gravity component
            gravity.velocity.y -= GRAVITY * PLAYER_FALL_MULTIPLIER;
        }
    }
}

fn add_player_velocity_system(mut query: Query<(&mut VelocityMap, &PlayerMovement)>) {
    for (mut vel_map, player) in query.iter_mut() {
        if let Some(vel) = vel_map.get_mut(player.vel_id) {
            *vel = player.velocity;
        } else {
            panic!("Players velocity not inside map, you forgot to register");
        }
    }
}

#[derive(Component, Debug)]
pub struct MouseCursor;

/// Moves the cursor
pub fn move_cursor_system(
    mut cursor_query: Query<&mut Transform, With<MouseCursor>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    windows: Res<Windows>,
) {
    const MOUSE_Z_POS: f32 = 7.0;

    for mut cursor_transform in cursor_query.iter_mut() {
        match camera_query.get_single() {
            Ok((camera, camera_transform)) => {
                let win = windows.get_primary().unwrap_or_else(|| panic!("No window"));
                if let Some(cursor_pos) = win.cursor_position() {
                    let window_size = Vec2::new(win.height(), win.height());
                    let ndc = (cursor_pos / window_size) * 2.0 - Vec2::ONE;
                    let ndc_to_world =
                        camera_transform.compute_matrix() * camera.projection_matrix().inverse();
                    let world_pos = {
                        let mut world_pos = ndc_to_world.project_point3(ndc.extend(-1.0));
                        world_pos.z = MOUSE_Z_POS;
                        world_pos
                    };

                    cursor_transform.translation = world_pos;
                }
            }
            Err(e) => {
                panic!(
                    "Multiple Cameras active, only one camera may be active at a time: {}",
                    e
                );
            }
        }
    }
}
