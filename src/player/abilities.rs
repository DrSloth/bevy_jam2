//! Implemen
//!
//! ## Terms
//! - Skill => The actual usable skill like dashing/double jump etc.
//! - Ability => An ability like Fire/Earth
//!     - The complete system of collecting iteGravity using skills
//! - Item => The collectible item which is stored in the invetory as ability

mod skills;

pub mod collectibles;

pub use skills::*;

use bevy::{
    ecs::system::EntityCommands,
    prelude::*,
    render::once_cell::sync::Lazy,
    sprite::collide_aabb,
    utils::{HashMap, Instant},
};
use std::{
    any::TypeId,
    fmt::{self, Debug, Formatter},
    time::Duration,
};

use super::MouseCursor;
use crate::{
    collision::{BreakableCollider, Collider},
    combat::Projectile,
    physics::VelocityMap,
};

// NOTE this would be nice if it was const (phf_map)
pub static ABILITY_MAP: Lazy<HashMap<AbilityItem, AbilityDescriptor>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(AbilityItem::Fire, PlayerDash::ability_descriptor());
    map.insert(AbilityItem::Earth, PlayerShoot::ability_descriptor());
    map.insert(AbilityItem::Steam, PlayerDoubleJump::ability_descriptor());
    map.insert(AbilityItem::Stone, PlayerCrouch::ability_descriptor());

    map
});

#[derive(Debug, serde::Deserialize, serde::Serialize, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum AbilityItem {
    Fire,
    Earth,
    Water,
    Steam,
    Stone,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AbilityId(TypeId);

pub trait Ability: Component + Default + Sized + 'static {
    fn ability_id() -> AbilityId {
        AbilityId(TypeId::of::<Self>())
    }

    fn unequip(player: &mut EntityCommands, inventory: &mut PlayerInventory) {
        player.remove::<Self>();
        inventory.unequip(Self::ability_id());
    }

    fn equip(player: &mut EntityCommands, inventory: &mut PlayerInventory, equip_slot: EquipSlot) {
        player.insert(Self::default());
        inventory.equip(Self::ability_id(), equip_slot);
    }

    fn ability_descriptor() -> AbilityDescriptor {
        AbilityDescriptor {
            id: Self::ability_id(),
            unequip: Self::unequip,
            equip: Self::equip,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AbilityDescriptor {
    id: AbilityId,
    unequip: fn(&mut EntityCommands, &mut PlayerInventory),
    equip: fn(&mut EntityCommands, &mut PlayerInventory, equip_slot: EquipSlot),
}

impl AbilityDescriptor {
    pub fn id(&self) -> &AbilityId {
        &self.id
    }

    #[allow(dead_code)] // NOTE will be used later
    pub fn unequip(&self, entity: &mut EntityCommands, inventory: &mut PlayerInventory) {
        (self.unequip)(entity, inventory);
    }

    pub fn equip(
        &self,
        entity: &mut EntityCommands,
        inventory: &mut PlayerInventory,
        equip_slot: EquipSlot,
    ) {
        (self.equip)(entity, inventory, equip_slot);
    }

    #[allow(dead_code)] // NOTE will be used later
    pub fn is_none(&self) -> bool {
        self.id() == &NoneAbility::ability_id()
    }
}

impl Debug for AbilityDescriptor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "AbilityDescriptor({:?})", self.id)
    }
}

#[derive(Component, Debug, Default)]
pub struct NoneAbility;

impl Ability for NoneAbility {}

#[derive(Debug, Component)]
pub struct PlayerInventory(AbilityId, AbilityId);

impl PlayerInventory {
    // pub fn contains<T: Ability>(&self) -> bool {
    //     let id = T::ability_id();
    //     id == self.0 || id == self.1
    // }

    #[allow(dead_code)] // Used when testing
    pub fn new() -> Self {
        Self::new_with::<NoneAbility, NoneAbility>()
    }

    pub fn new_with<T: Ability, U: Ability>() -> Self {
        Self(T::ability_id(), U::ability_id())
    }

    // pub fn equip_ability<T: Ability>(&mut self, slot: EquipSlot) {
    //     self.equip(T::ability_id(), slot)
    // }

    pub fn equip(&mut self, id: AbilityId, slot: EquipSlot) {
        match slot {
            EquipSlot::Left => self.0 = id,
            EquipSlot::Right => self.1 = id,
        }
    }

    pub fn unequip(&mut self, id: AbilityId) {
        if self.0 == id {
            self.0 = NoneAbility::ability_id();
        }

        if self.1 == id {
            self.1 = NoneAbility::ability_id();
        }
    }

    pub fn is_equipped_at<T: Ability>(&self, slot: EquipSlot) -> bool {
        match slot {
            EquipSlot::Left => T::ability_id() == self.0,
            EquipSlot::Right => T::ability_id() == self.1,
        }
    }

    pub fn get_equipped_at<T: Ability>(&self) -> Option<EquipSlot> {
        let id = T::ability_id();

        if self.0 == id {
            Some(EquipSlot::Left)
        } else if self.1 == id {
            Some(EquipSlot::Right)
        } else {
            None
        }
    }

    pub fn first_free_slot(&self) -> Option<EquipSlot> {
        if self.0 == NoneAbility::ability_id() {
            Some(EquipSlot::Left)
        } else if self.1 == NoneAbility::ability_id() {
            Some(EquipSlot::Right)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy)]
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

    pub fn to_mouse_btn(self) -> MouseButton {
        match self {
            Self::Left => MouseButton::Left,
            Self::Right => MouseButton::Right,
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
    const PLAYER_PROJECTILE_SPEED: f32 = 5.5;
    const PLAYER_SHOOT_INTERVAL: Duration = Duration::from_millis(450);
    const PLAYER_SHOT_SIZE: f32 = 4.0;

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
                let size = Vec2::splat(PLAYER_SHOT_SIZE);
                commands
                    .spawn_bundle(SpriteBundle {
                        sprite: Sprite {
                            color: Color::rgb(0.0, 1.0, 1.0),
                            custom_size: Some(size),
                            ..Default::default()
                        },
                        transform: Transform::from_translation(player_transform.translation),
                        ..Default::default()
                    })
                    .insert(VelocityMap::new())
                    .insert(PlayerShotProjectile::new(size))
                    .insert(projectile);
            }
        }
    }
}

/// Component for projectiles that can break breakable walls
#[derive(Debug, Component)]
pub struct PlayerShotProjectile {
    pub size: Vec2,
    creation_time: Instant,
}

impl PlayerShotProjectile {
    fn new(size: Vec2) -> Self {
        Self {
            size,
            creation_time: Instant::now(),
        }
    }
}

/// System that destroys breakable colliders with player projectiles
pub fn player_shot_collision_system(
    mut commands: Commands,
    shot_query: Query<(&Transform, &PlayerShotProjectile, Entity)>,
    breakable_wall_query: Query<(&Transform, &Collider, Option<&BreakableCollider>, Entity)>,
) {
    'outer: for (shot_trans, shot, shot_entity) in shot_query.iter() {
        for (wall_trans, wall_coll, breakable, wall_entity) in breakable_wall_query.iter() {
            if collide_aabb::collide(
                shot_trans.translation,
                shot.size,
                wall_trans.translation,
                wall_coll.size,
            )
            .is_some()
            {
                commands.entity(shot_entity).despawn();
                if breakable.is_some() {
                    commands.entity(wall_entity).despawn();
                }
                continue 'outer;
            }
        }

        if shot.creation_time.elapsed() > Duration::from_secs(30) {
            commands.entity(shot_entity).despawn();
        }
    }
}
