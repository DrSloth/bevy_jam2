use std::any::TypeId;

use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::{
    physics::Gravity,
    player::{
        abilities::{Ability, AbilityId, EquipSlot, PlayerInventory},
        PlayerCollisionEvent, PlayerMovement,
    },
};

#[derive(Component, Debug, Default)]
pub struct PlayerWallJump {
    wall_side: Option<WallSide>,
    wall_jumped: bool,
}

impl Ability for PlayerWallJump {
    const ABILITY_ID: AbilityId = AbilityId::Water;
}

/// On which side of a wall we are
#[derive(Debug, Clone, Copy)]
enum WallSide {
    Left,
    Right,
}

pub fn player_wall_jump_system(
    mouse_input: ResMut<Input<MouseButton>>,
    mut query: Query<(
        &mut Gravity,
        &mut PlayerMovement,
        &mut PlayerWallJump,
        &PlayerInventory,
    )>,
) {
    const WALL_JUMP_POWER: Vec2 = Vec2::new(4.0, 7.0);

    for (mut grav, mut player, mut wall_jump, inv) in query.iter_mut() {
        if let Some(mouse_button) = inv
            .get_equipped_at::<PlayerWallJump>()
            .map(EquipSlot::to_mouse_btn)
        {
            if mouse_input.just_pressed(mouse_button) {
                match wall_jump.wall_side {
                    Some(WallSide::Right) => {
                        grav.velocity = Vec2::ZERO;
                        player.velocity = WALL_JUMP_POWER;
                        wall_jump.wall_jumped = true;
                        player
                            .move_forbid_set
                            .insert(TypeId::of::<PlayerWallJump>());
                    }
                    Some(WallSide::Left) => {
                        grav.velocity = Vec2::ZERO;
                        player.velocity.y = WALL_JUMP_POWER.y;
                        player.velocity.x = -WALL_JUMP_POWER.x;
                        wall_jump.wall_jumped = true;
                        player
                            .move_forbid_set
                            .insert(TypeId::of::<PlayerWallJump>());
                    }
                    None => (),
                }

                wall_jump.wall_side = None;
            }
        }
    }
}

pub fn wall_jump_collision_system(
    mut events: EventReader<PlayerCollisionEvent>,
    mut player_query: Query<(&mut PlayerWallJump, &mut PlayerMovement)>,
) {
    // TODO this currently handles all active players as one (currently only one)
    let mut new_wall_side = None;
    let mut landed = false;
    for evt in events.iter() {
        new_wall_side = match evt.collision_side {
            Collision::Left => Some(WallSide::Left),
            Collision::Right => Some(WallSide::Right),
            Collision::Top => {
                landed = true;
                None
            }
            _ => None,
        };
    }

    for (mut wall_jump, mut player) in player_query.iter_mut() {
        if wall_jump.wall_jumped && landed {
            player
                .move_forbid_set
                .remove(&TypeId::of::<PlayerWallJump>());
            wall_jump.wall_jumped = false;
        }
        wall_jump.wall_side = new_wall_side;
    }
}
