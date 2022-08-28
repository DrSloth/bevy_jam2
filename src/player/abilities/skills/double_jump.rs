use bevy::prelude::*;

use crate::{
    physics::Gravity,
    player::{
        abilities::{Ability, EquipSlot, PlayerInventory},
        PlayerLandEvent, PlayerMovement,
    },
};

#[derive(Component, Debug)]
pub struct PlayerDoubleJump {
    can_dbl_jump: bool,
}

impl Default for PlayerDoubleJump {
    fn default() -> Self {
        PlayerDoubleJump { can_dbl_jump: true }
    }
}

impl Ability for PlayerDoubleJump {}

pub fn player_double_jump_system(
    mouse_input: ResMut<Input<MouseButton>>,
    mut query: Query<(
        &mut Gravity,
        &mut PlayerDoubleJump,
        &mut PlayerMovement,
        &PlayerInventory,
    )>,
) {
    const DOUBLE_JUMP_POWER: f32 = 10.5;
    // const DOUBLE_JUMP_POWER: f32 = 18.5;

    for (mut grav, mut jump, mut player_mov, inv) in query.iter_mut() {
        if let Some(mouse_button) = inv
            .get_equipped_at::<PlayerDoubleJump>()
            .map(EquipSlot::to_mouse_btn)
        {
            if mouse_input.just_pressed(mouse_button) && jump.can_dbl_jump {
                grav.velocity = Vec2::ZERO;
                player_mov.velocity.y = DOUBLE_JUMP_POWER;
                player_mov.can_jump = false;
                jump.can_dbl_jump = false;
            }
        }
    }
}

pub fn double_jump_land_system(
    mut land_events: EventReader<PlayerLandEvent>,
    mut query: Query<&mut PlayerDoubleJump>,
) {
    for PlayerLandEvent {
        player_entity: ent, ..
    } in land_events.iter()
    {
        if let Ok(mut dbl_jump) = query.get_mut(*ent) {
            dbl_jump.can_dbl_jump = true;
        }
    }
}
