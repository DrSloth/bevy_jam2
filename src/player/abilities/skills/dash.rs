use bevy::{prelude::*, utils::Instant};
use std::{any::TypeId, time::Duration};

use crate::{
    physics::Gravity,
    player::{
        abilities::{Ability, EquipSlot, PlayerInventory},
        PlayerMovement,
    },
};

const PLAYER_DASH_SPEED: f32 = 8.0;
const PLAYER_DASH_INTERVAL: Duration = Duration::from_millis(1500);
const PLAYER_RUN_EPSILON: f32 = 0.2;
const PLAYER_DASH_DURATION: Duration = Duration::from_millis(150);

#[derive(Component, Debug, Default)]
pub struct PlayerDash {
    pub(crate) last_dash: Option<Instant>,
}

impl Ability for PlayerDash {}

pub fn player_dash_system(
    mouse_input: ResMut<Input<MouseButton>>,
    mut player_query: Query<(
        &mut PlayerDash,
        &mut PlayerMovement,
        &mut Gravity,
        &PlayerInventory,
    )>,
) {
    for click in mouse_input.get_pressed() {
        for (mut player_dash, _, _, inv) in player_query.iter_mut() {
            if !EquipSlot::from_mouse_btn(*click)
                .map_or(false, |slot| inv.is_equipped_at::<PlayerDash>(slot))
            {
                continue;
            }

            if player_dash.last_dash.is_some() {
                continue;
            }

            player_dash.last_dash = Some(Instant::now());
        }
    }

    for (mut player_dash, mut player, mut gravity, _) in player_query.iter_mut() {
        match player_dash.last_dash {
            None => {
                continue;
            }
            Some(last_dash) if last_dash.elapsed() > PLAYER_DASH_INTERVAL => {
                player.move_forbid_set.remove(&TypeId::of::<PlayerDash>());
                player_dash.last_dash = None;
                continue;
            }
            Some(last_dash) if last_dash.elapsed() > PLAYER_DASH_DURATION => {
                player.move_forbid_set.remove(&TypeId::of::<PlayerDash>());
                continue;
            }
            Some(_) => (),
        }

        if player.velocity.x < PLAYER_RUN_EPSILON && player.velocity.x > -PLAYER_RUN_EPSILON {
            continue;
        }

        if player.velocity.x > PLAYER_RUN_EPSILON {
            player.velocity.x = PLAYER_DASH_SPEED;
        } else if player.velocity.x < -PLAYER_RUN_EPSILON {
            player.velocity.x = -PLAYER_DASH_SPEED;
        }

        player.velocity.y = 0.0;
        gravity.velocity.y = 0.0;
        player.move_forbid_set.insert(TypeId::of::<PlayerDash>());
    }
}
