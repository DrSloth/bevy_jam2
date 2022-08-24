use bevy::{ecs::world::EntityMut, prelude::*, utils::Instant};
use std::{any::TypeId, time::Duration};

use super::{MouseCursor, PlayerMovement};
use crate::{
    combat::Projectile,
    physics::{Gravity, VelocityMap},
};

#[derive(Clone, PartialEq, Debug)]
pub struct AbilityId(TypeId);

pub trait Ability: Component + Sized + 'static {
    fn ability_id() -> AbilityId {
        AbilityId(TypeId::of::<Self>())
    }

    fn unequip(entity: &mut EntityMut) {
        entity.remove::<Self>();
    }
}

#[derive(Component, Debug)]
pub struct NoneAbility;

impl Ability for NoneAbility {}

#[derive(Debug, Component)]
pub struct PlayerInventory(AbilityId, AbilityId);

impl PlayerInventory {
    // pub fn contains<T: Ability>(&self) -> bool {
    //     let id = T::ability_id();
    //     id == self.0 || id == self.1
    // }

    // pub fn new() -> Self {
    //     Self::new_with::<NoneAbility, NoneAbility>()
    // }

    pub fn new_with<T: Ability, U: Ability>() -> Self {
        Self(T::ability_id(), U::ability_id())
    }

    // pub fn equip_ability<T: Ability>(&mut self, slot: EquipSlot) {
    //     self.equip(T::ability_id(), slot)
    // }

    // pub fn equip(&mut self, id: AbilityId, slot: EquipSlot) {
    //     match slot {
    //         EquipSlot::Left => self.0 = id,
    //         EquipSlot::Right => self.1 = id,
    //     }
    // }

    pub fn is_equipped_at<T: Ability>(&self, slot: EquipSlot) -> bool {
        match slot {
            EquipSlot::Left => T::ability_id() == self.0,
            EquipSlot::Right => T::ability_id() == self.1,
        }
    }
}

pub enum EquipSlot {
    Left,
    Right,
}

impl EquipSlot {
    fn from_mouse_btn(btn: MouseButton) -> Option<Self> {
        match btn {
            MouseButton::Left => Some(EquipSlot::Left),
            MouseButton::Right => Some(EquipSlot::Right),
            _ => None,
        }
    }
}

/// The shooting ability, currently the `Earth` ability
#[derive(Debug, Component)]
pub struct PlayerShoot {
    last_shot: Instant,
}

impl Ability for PlayerShoot {}

impl Default for PlayerShoot {
    fn default() -> Self {
        Self {
            last_shot: Instant::now(),
        }
    }
}

pub fn player_shoot_system(
    mut commands: Commands,
    mouse_input: ResMut<Input<MouseButton>>,
    mut player_query: Query<(&Transform, &PlayerInventory, &mut PlayerShoot)>,
    cursor_query: Query<&Transform, With<MouseCursor>>,
) {
    const PLAYER_PROJECTILE_SPEED: f32 = 20.0;
    const PLAYER_SHOOT_INTERVAL: Duration = Duration::from_millis(450);

    for click in mouse_input.get_pressed() {
        for (player_transform, inv, mut player_shoot) in player_query.iter_mut() {
            if !EquipSlot::from_mouse_btn(*click)
                .map_or(false, |slot| inv.is_equipped_at::<PlayerShoot>(slot))
            {
                continue;
            }

            if player_shoot.last_shot.elapsed() < PLAYER_SHOOT_INTERVAL {
                continue;
            }

            player_shoot.last_shot = Instant::now();

            for cursor in cursor_query.iter() {
                let direction = -(player_transform.translation - cursor.translation)
                    .normalize()
                    .truncate();
                let projectile = Projectile {
                    speed: PLAYER_PROJECTILE_SPEED,
                    direction,
                    vel_id: None,
                };
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.0, 1.0, 1.0),
                            custom_size: Some(Vec2::new(15.0, 15.0)),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(player_transform.translation),
                        ..Default::default()
                    })
                    .insert(VelocityMap::new())
                    .insert(projectile);
            }
        }
    }
}

//TODO stop making the vel_ids options, require a vel_map/vel_id when creating

#[derive(Component, Debug)]
pub struct PlayerDash {
    last_dash: Instant,
    pressed: bool,
}

impl Ability for PlayerDash {}

impl Default for PlayerDash {
    fn default() -> Self {
        Self {
            last_dash: Instant::now(),
            pressed: false,
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
    const PLAYER_DASH_SPEED: f32 = 22.0;
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

            if player_dash.last_dash.elapsed() < PLAYER_DASH_INTERVAL {
                continue;
            }

            player_dash.last_dash = Instant::now();
            player_dash.pressed = true;
        }
    }

    for (mut vel_map, player_dash, mut player, gravity, _) in player_query.iter_mut() {
        let elapsed = player_dash.last_dash.elapsed();
        if elapsed > PLAYER_DASH_DURATION || !player_dash.pressed {
            player.can_move = true;
            continue;
        }

        if let (Some(player_vel_id), Some(gravity)) = (player.vel_id, gravity.vel_id) {
            if let (Some(mut player_vel), Some(mut grav_vel)) =
                (vel_map.get(player_vel_id), vel_map.get(gravity))
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

                vel_map.set(player_vel_id, player_vel);
                vel_map.set(gravity, grav_vel);
            }
        }
    }
}
