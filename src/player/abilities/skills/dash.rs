use bevy::{prelude::*, utils::Instant};
use std::time::Duration;

use crate::{
    physics::Gravity,
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
        &mut PlayerDash,
        &mut PlayerMovement,
        &mut Gravity,
        &PlayerInventory,
    )>,
) {
    const PLAYER_DASH_SPEED: f32 = 8.0;
    const PLAYER_DASH_INTERVAL: Duration = Duration::from_millis(1500);
    const PLAYER_RUN_EPSILON: f32 = 0.2;
    const PLAYER_DASH_DURATION: Duration = Duration::from_millis(150);

    for click in mouse_input.get_pressed() {
        for (mut player_dash, _, _, inv) in player_query.iter_mut() {
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

    for (player_dash, mut player, mut gravity, _) in player_query.iter_mut() {
        let elapsed = player_dash.last_dash.elapsed();
        if elapsed > PLAYER_DASH_DURATION || !player_dash.dashed_once {
            player.can_move = true;
            continue;
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
        player.can_move = false;
    }
}
