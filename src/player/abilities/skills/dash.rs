use bevy::{prelude::*, utils::Instant};
use std::time::Duration;

use crate::{
    physics::{Gravity, VelocityMap},
    player::{
        abilities::{Ability, EquipSlot, PlayerInventory},
        PlayerMovement,
    },
};

#[derive(Component, Debug)]
pub struct PlayerDash {
    pub(crate) last_dash: Instant,
    pub(crate) dashed_once: bool,
}

impl Ability for PlayerDash {}

impl Default for PlayerDash {
    fn default() -> Self {
        Self {
            last_dash: Instant::now(),
            dashed_once: false,
        }
    }
}

pub fn player_dash_system(
    mouse_input: ResMut<Input<MouseButton>>,
    mut player_query: Query<(
        &mut VelocityMap,
        &mut PlayerDash,
        &mut PlayerMovement,
        &Gravity,
        &PlayerInventory,
    )>,
) {
    const PLAYER_DASH_SPEED: f32 = 8.0;
    const PLAYER_DASH_INTERVAL: Duration = Duration::from_millis(1500);
    const PLAYER_RUN_EPSILON: f32 = 0.2;
    const PLAYER_DASH_DURATION: Duration = Duration::from_millis(150);

    for click in mouse_input.get_pressed() {
        for (_, mut player_dash, _, _, inv) in player_query.iter_mut() {
            if !EquipSlot::from_mouse_btn(*click)
                .map_or(false, |slot| inv.is_equipped_at::<PlayerDash>(slot))
            {
                continue;
            }

            if player_dash.last_dash.elapsed() < PLAYER_DASH_INTERVAL && player_dash.dashed_once {
                continue;
            }

            player_dash.last_dash = Instant::now();
            player_dash.dashed_once = true;
        }
    }

    for (mut vel_map, player_dash, mut player, gravity, _) in player_query.iter_mut() {
        let elapsed = player_dash.last_dash.elapsed();
        if elapsed > PLAYER_DASH_DURATION || !player_dash.dashed_once {
            player.can_move = true;
            continue;
        }

        if let (Some(mut player_vel), Some(mut grav_vel)) =
            (vel_map.get(player.vel_id), vel_map.get(gravity.vel_id))
        {
            if player_vel.x < PLAYER_RUN_EPSILON && player_vel.x > -PLAYER_RUN_EPSILON {
                continue;
            }

            if player_vel.x > PLAYER_RUN_EPSILON {
                player_vel.x = PLAYER_DASH_SPEED;
            } else if player_vel.x < -PLAYER_RUN_EPSILON {
                player_vel.x = -PLAYER_DASH_SPEED;
            }

            player_vel.y = 0.0;
            grav_vel.y = 0.0;
            player.can_move = false;

            if let Err(e) = vel_map
                .set(player.vel_id, player_vel)
                .and_then(|_| vel_map.set(gravity.vel_id, grav_vel))
            {
                panic!("{} -> The velocity map was reset while ids were held", e)
            }
        }
    }
}
