use bevy::prelude::*;

use crate::{
    collision::{BreakableCollider, Collider},
    physics::{Gravity, GRAVITY, GRAVITY_MAX},
    player::{
        abilities::{Ability, PlayerInventory},
        PlayerLandEvent, PlayerMovement,
    },
    PLAYER_SIZE,
};

#[derive(Component, Debug, Default)]
pub struct PlayerCrouch {
    state: CrouchState,
}

impl Ability for PlayerCrouch {}

#[derive(Debug, Clone, Copy, PartialEq)]
enum CrouchState {
    NotCrouching,
    Airborne,
    Grounded,
}

impl Default for CrouchState {
    fn default() -> Self {
        Self::NotCrouching
    }
}

pub fn player_crouch_system(
    mut query: Query<(
        &mut Gravity,
        &mut PlayerCrouch,
        &mut PlayerMovement,
        &mut Collider,
        &PlayerInventory,
    )>,
    mouse_input: ResMut<Input<MouseButton>>,
) {
    const CROUCH_FALL_SPEED: f32 = GRAVITY * 3.0;
    const MAX_CROUCH_FALL_SPEED: f32 = GRAVITY_MAX * 3.0;
    const CROUCH_SIZE: f32 = 4.0;
    // const CROUCH_SIZE: f32 = 16.0;

    for (mut grav, mut crouch, mut player_move, mut moving_collider, inv) in query.iter_mut() {
        if let Some(equip_slot) = inv.get_equipped_at::<PlayerCrouch>() {
            loop {
                match crouch.state {
                    CrouchState::NotCrouching => {
                        if mouse_input.just_pressed(equip_slot.to_mouse_btn()) {
                            crouch.state = CrouchState::Airborne;
                            moving_collider.size = Vec2::splat(CROUCH_SIZE);
                            player_move.velocity.y = 0.0;
                        } else {
                            moving_collider.size = Vec2::splat(PLAYER_SIZE);
                            break;
                        }
                    }
                    CrouchState::Airborne => {
                        moving_collider.size = Vec2::splat(CROUCH_SIZE);
                        grav.velocity = Vec2::ZERO;
                        player_move.velocity.y =
                            (player_move.velocity.y - CROUCH_FALL_SPEED).max(MAX_CROUCH_FALL_SPEED);

                        player_move.velocity.x = 0.0;
                        break;
                    }
                    CrouchState::Grounded => {
                        moving_collider.size = Vec2::splat(CROUCH_SIZE);
                        player_move.can_jump = false;
                        player_move.velocity.y = 0.0;
                        if grav.velocity.y < -GRAVITY * 2.0 {
                            crouch.state = CrouchState::Airborne;
                            continue;
                        }

                        if mouse_input.just_pressed(equip_slot.to_mouse_btn()) {
                            crouch.state = CrouchState::NotCrouching;
                            moving_collider.size = Vec2::splat(PLAYER_SIZE);
                            player_move.can_jump = true;
                            break;
                        }

                        player_move.velocity.x /= 2.0;
                        break;
                    }
                }
            }
        }
    }
}

pub fn crouch_collision_system(
    mut commands: Commands,
    mut query: Query<&mut PlayerCrouch>,
    breakables_query: Query<Entity, With<BreakableCollider>>,
    mut event_reader: EventReader<PlayerLandEvent>,
) {
    let mut landed_this_frame = false;
    for evt in event_reader.iter() {
        if let Ok(mut crouch) = query.get_mut(evt.player_entity) {
            if CrouchState::Airborne == crouch.state || landed_this_frame {
                if let Ok(ent) = breakables_query.get(evt.ground_entity) {
                    commands.entity(ent).despawn();
                }
                crouch.state = CrouchState::Grounded;
                landed_this_frame = true;
            }
        }
    }
}
