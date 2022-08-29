pub mod abilities;

use std::any::TypeId;
use std::time::Duration;

use bevy::utils::{HashSet, Instant};
use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::asset_loaders::cache::AssetCache;
use crate::collision::{Collider, CollisionFilter};
use crate::POST_COLLISION_STAGE;
use crate::{
    asset_loaders::EmbeddedAssets,
    camera::FollowedByCamera,
    collision::{CollisionEvent, MoveableCollider},
    physics::{Gravity, VelocityId, VelocityMap, GRAVITY, VEL_SYSTEM_STAGE},
    LATE_UPDATE_STAGE, PLAYER_SIZE,
};
use abilities::{collectibles, PlayerInventory};

use self::abilities::{
    crouch_collision_system, double_jump_land_system, player_crouch_system,
    player_wall_jump_system, wall_jump_collision_system, PlayerDash, PlayerShoot, PlayerWallJump,
};

pub const PLAYER_SPAWN_STAGE: &str = "play_spawn";

#[derive(Debug)]
pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_stage_before(
            VEL_SYSTEM_STAGE,
            PLAYER_SPAWN_STAGE,
            SystemStage::parallel(),
        )
        .add_startup_system(player_setup_system)
        .add_system(player_input_system)
        .add_system(player_jump_system)
        .add_system(player_collision_system)
        .add_system(abilities::player_shoot_system)
        .add_system(collectibles::collect_ability_system)
        .add_system(abilities::player_shot_collision_system)
        .add_system(player_turn_system)
        .add_system_to_stage(CoreStage::PreUpdate, move_cursor_system)
        .add_system_to_stage(LATE_UPDATE_STAGE, abilities::player_dash_system)
        .add_system_to_stage(LATE_UPDATE_STAGE, abilities::player_double_jump_system)
        .add_system_to_stage(POST_COLLISION_STAGE, double_jump_land_system)
        .add_system_to_stage(LATE_UPDATE_STAGE, player_crouch_system)
        .add_system_to_stage(POST_COLLISION_STAGE, crouch_collision_system)
        .add_system_to_stage(POST_COLLISION_STAGE, wall_jump_collision_system)
        .add_system_to_stage(LATE_UPDATE_STAGE, player_wall_jump_system)
        .add_system_to_stage(LATE_UPDATE_STAGE, player_fall_system)
        .add_system_to_stage(VEL_SYSTEM_STAGE, add_player_velocity_system)
        .add_system_to_stage(LATE_UPDATE_STAGE, player_spawn_help_system)
        .add_event::<PlayerLandEvent>()
        .add_event::<PlayerCollisionEvent>()
        .add_event::<JumpEvent>();
    }
}

fn player_setup_system(
    mut commands: Commands,
    mut asset_cache: ResMut<AssetCache<EmbeddedAssets>>,
    mut assets: ResMut<Assets<Image>>,
) {
    let texture = asset_cache
        .load_image(&mut assets, "sprites/character/movement/idle.png")
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
                translation: Vec3::new(10.0 * PLAYER_SIZE, 4.0 * PLAYER_SIZE, 0.0),
                // translation: Vec3::new(84.0, 197.0, 0.0),
                // translation: Vec3::new(404.0, 12.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(FollowedByCamera)
        .insert(PlayerMovement::new_in(&mut vel_map))
        .insert(Gravity::new_in(&mut vel_map))
        .insert(PlayerSpawn::new(&mut vel_map))
        .insert(vel_map)
        .insert(PlayerInventory::new())
        .insert(PlayerInventory::new_with::<PlayerShoot, PlayerWallJump>())
        // .insert(PlayerCrouch::default())
        // .insert(PlayerDoubleJump::default())
        .insert(PlayerDash::default())
        .insert(PlayerShoot::default())
        .insert(PlayerWallJump::default())
        .insert(Collider {
            size: Vec2::new(PLAYER_SIZE / 1.2, PLAYER_SIZE),
            collision_offset: Vec2::new(PLAYER_SIZE / 8.5, PLAYER_SIZE / 3.0),
            filter: CollisionFilter::ALL,
        })
        .insert(MoveableCollider);
}

#[derive(Debug)]
pub struct JumpEvent(pub Entity);

/// Component only added to the player character
#[derive(Component, Debug)]
pub struct PlayerMovement {
    vel_id: VelocityId,
    can_jump: bool,
    move_forbid_set: HashSet<TypeId>,
    pub velocity: Vec2,
}

impl PlayerMovement {
    /// Create a new player movement with the given `vel_id`
    pub fn new(vel_id: VelocityId) -> Self {
        Self {
            vel_id,
            can_jump: true,
            move_forbid_set: HashSet::default(),
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
    const SPEED: f32 = 3.5;

    for (mut player, entity) in player_query.iter_mut() {
        if !player.move_forbid_set.is_empty() {
            continue;
        }

        player.velocity.x = 0.0;
        for key in kb_input.get_pressed() {
            match key {
                KeyCode::A => {
                    player.velocity.x += -SPEED;
                }
                KeyCode::D => {
                    player.velocity.x += SPEED;
                }
                KeyCode::Space => jump_event_writer.send(JumpEvent(entity)),
                _ => (),
            }
        }
    }
}

fn player_turn_system(
    mut player_query: Query<&mut Transform, With<PlayerMovement>>,
    mouse_cursor: Query<&Transform, (With<MouseCursor>, Without<PlayerMovement>)>,
) {
    for mut player in player_query.iter_mut() {
        for cursor in mouse_cursor.iter() {
            if cursor.translation.x < player.translation.x {
                player.rotation = Quat::from_axis_angle(Vec3::Y, 180.0f32.to_radians());
            } else {
                player.rotation = Quat::from_axis_angle(Vec3::Y, 0.0);
            }
        }
    }
}

pub fn player_jump_system(
    mut player_query: Query<(&mut PlayerMovement, &mut Gravity)>,
    mut jump_event_reader: EventReader<JumpEvent>,
) {
    const JUMP_POWER: f32 = 7.5;

    for JumpEvent(entity) in jump_event_reader.iter() {
        if let Ok((mut player_movement, mut grav)) = player_query.get_mut(*entity) {
            let falling = is_falling(grav.velocity.y);
            let can_jump = !falling && player_movement.can_jump;

            if can_jump {
                player_movement.velocity.y = JUMP_POWER;
                player_movement.can_jump = false;
                grav.velocity.y = 0.0;
            }
        }
    }
}

fn player_collision_system(
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut event_writer: EventWriter<PlayerLandEvent>,
    mut player_collision_writer: EventWriter<PlayerCollisionEvent>,
    mut player_query: Query<(&mut PlayerMovement, &Gravity, &VelocityMap, Entity)>,
) {
    for collision in collision_event_reader.iter() {
        // TODO do this more beautifully
        if player_query.get(collision.moving_entity).is_ok() {
            player_collision_writer.send(PlayerCollisionEvent {
                player_entity: collision.moving_entity,
                static_entity: collision.static_entity,
                collision_side: copy_collision(&collision.collision),
            });
        }

        if let Collision::Top = collision.collision {
            if let Ok((mut player, grav, vel_map, player_entity)) =
                player_query.get_mut(collision.moving_entity)
            {
                event_writer.send(PlayerLandEvent {
                    player_entity,
                    ground_entity: collision.static_entity,
                });
                let player_y_speed = player.velocity.y.abs();

                if let Some(grav_vel) = vel_map.get(grav.vel_id) {
                    if !is_falling(grav_vel.y) && player_y_speed < GRAVITY && !player.can_jump {
                        player.can_jump = true;
                    }
                }
            }
        } else if let Collision::Bottom = collision.collision {
            if let Ok((mut player, _grav, _vel_map, _ent)) =
                player_query.get_mut(collision.moving_entity)
            {
                player.velocity.y = 0.0;
            }
        }
    }
}

/// NOTE this is stupid
pub fn copy_collision(collision: &Collision) -> Collision {
    match collision {
        Collision::Left => Collision::Left,
        Collision::Right => Collision::Right,
        Collision::Top => Collision::Top,
        Collision::Bottom => Collision::Bottom,
        Collision::Inside => Collision::Inside,
    }
}

/// Specific event for the player landing/staying grounded
pub struct PlayerLandEvent {
    player_entity: Entity,
    ground_entity: Entity,
}

/// General collision event of player
pub struct PlayerCollisionEvent {
    pub player_entity: Entity,
    pub static_entity: Entity,
    /// The side of the static entity we collided with
    pub collision_side: Collision,
}

fn is_falling(grav_y_vel: f32) -> bool {
    grav_y_vel < -(GRAVITY * 3.0)
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

#[derive(Debug, Component)]
pub struct PlayerSpawn {
    vel_id: VelocityId,
    spawn_time: Option<Instant>,
}

impl PlayerSpawn {
    pub fn new(vel_map: &mut VelocityMap) -> Self {
        Self {
            vel_id: vel_map.register().0,
            spawn_time: None,
        }
    }

    pub fn spawn_from_bottom(&mut self) {
        self.spawn_time = Some(Instant::now());
    }
}

fn player_spawn_help_system(
    mut query: Query<(
        &PlayerSpawn,
        &mut Gravity,
        &mut PlayerMovement,
        &mut VelocityMap,
    )>,
) {
    const SPAWN_VEL_DUR: Duration = Duration::from_millis(400);
    const SPAWN_VEL: Vec2 = Vec2::new(-4.0, 4.0);

    for (spawn, mut grav, mut mov, mut vel_map) in query.iter_mut() {
        if let Some(spawn_time) = spawn.spawn_time {
            if let Some(vel) = vel_map.get_mut(spawn.vel_id) {
                if spawn_time.elapsed() < SPAWN_VEL_DUR {
                    *vel = SPAWN_VEL;
                    mov.velocity = Vec2::ZERO;
                    grav.velocity = Vec2::ZERO;
                } else {
                    *vel = Vec2::ZERO;
                }
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct MouseCursor;

/// Moves the cursor
pub fn move_cursor_system(
    mut cursor_query: Query<&mut Transform, With<MouseCursor>>,
    camera_query: Query<(&Camera, &Transform), Without<MouseCursor>>,
    windows: Res<Windows>,
) {
    const MOUSE_Z_POS: f32 = 7.0;

    for mut cursor_transform in cursor_query.iter_mut() {
        match camera_query.get_single() {
            Ok((camera, camera_transform)) => {
                let win = if let Some(win) = windows.get_primary() {
                    win
                } else {
                    return;
                };
                if let Some(cursor_pos) = win.cursor_position() {
                    let window_size = Vec2::new(win.width(), win.height());
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
