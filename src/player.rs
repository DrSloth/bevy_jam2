use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::{
    collision::CollisionEvent,
    physics::{Gravity, VelocityId, VelocityMap, GRAVITY},
};

#[derive(Debug)]
pub struct JumpEvent(pub Entity);

/// Component only added to the player character
#[derive(Component, Debug)]
pub struct PlayerMovement {
    pub(crate) vel_id: Option<VelocityId>,
    can_jump: bool,
}

impl Default for PlayerMovement {
    fn default() -> Self {
        Self {
            vel_id: None,
            can_jump: true,
        }
    }
}

impl PlayerMovement {
    /// Create a new player movement withou a velocity id
    pub fn new() -> Self {
        Self::default()
    }
}

/// System to move the player with input
pub fn player_input_system(
    mut player_query: Query<(&mut VelocityMap, &mut PlayerMovement, Entity)>,
    mut jump_event_writer: EventWriter<JumpEvent>,
    kb_input: ResMut<Input<KeyCode>>,
) {
    const SPEED: f32 = 10.0;

    for (mut velocity_map, mut player, entity) in player_query.iter_mut() {
        let vel = if let Some(vel) = player.vel_id.and_then(|id| velocity_map.get_mut(id)) {
            vel
        } else {
            let (id, vel) = velocity_map.register();
            player.vel_id = Some(id);
            vel
        };

        vel.x = 0.0;
        for key in kb_input.get_pressed() {
            match key {
                KeyCode::A => vel.x = -SPEED,
                KeyCode::D => vel.x = SPEED,
                KeyCode::Space => jump_event_writer.send(JumpEvent(entity)),
                _ => (),
            }
        }
    }
}

pub fn player_jump_system(
    mut player_query: Query<(&mut VelocityMap, &mut PlayerMovement, &Gravity)>,
    mut jump_event_reader: EventReader<JumpEvent>,
) {
    const JUMP_POWER: f32 = 24.0;

    for JumpEvent(entity) in jump_event_reader.iter() {
        if let Ok((mut velocity_map, mut player_movement, grav)) = player_query.get_mut(*entity) {
            let falling = is_falling(grav, &*velocity_map);
            let can_jump = !falling && player_movement.can_jump;

            if can_jump {
                if let Some(vel) = player_movement
                    .vel_id
                    .and_then(|id| velocity_map.get_mut(id))
                {
                    vel.y = JUMP_POWER;
                    player_movement.can_jump = false;
                }
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
                let player_y_speed = (player
                    .vel_id
                    .and_then(|id| vel_map.get(id))
                    .unwrap_or(Vec2::ZERO)
                    .y)
                    .abs();
                if !is_falling(grav, vel_map) && player_y_speed < GRAVITY && !player.can_jump {
                    player.can_jump = true;
                }
            }
        }
    }
}

fn is_falling(grav: &Gravity, vel_map: &VelocityMap) -> bool {
    grav.vel_id()
        .and_then(|id| vel_map.get(id))
        .map_or(true, |v| v.y < -(GRAVITY * 3.0))
}

// Makes the player slow down while falling
pub fn player_fall_system(mut player_query: Query<(&mut VelocityMap, &PlayerMovement, &Gravity)>) {
    for (mut velocity_map, player, grav) in player_query.iter_mut() {
        if let (Some(player_id), Some(grav_id)) = (player.vel_id, *grav.vel_id()) {
            if let (Some(mut player_vel), Some(mut gravity_vel)) = (
                player.vel_id.and_then(|id| velocity_map.get(id)),
                grav.vel_id().and_then(|id| velocity_map.get(id)),
            ) {
                if player_vel.y > 0.0 {
                    player_vel.y += gravity_vel.y;
                    gravity_vel = Vec2::ZERO;
                } else if gravity_vel.y < -GRAVITY {
                    // TODO maybe this should be a gravity scale in the gravity component
                    gravity_vel.y -= GRAVITY * 2.0;
                }

                velocity_map.set(player_id, player_vel);
                velocity_map.set(grav_id, gravity_vel);
            }
        }
    }
}
