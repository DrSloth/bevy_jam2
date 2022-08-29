mod slime;

use std::{
    fmt::Debug,
    ops::{Sub, SubAssign},
};

use bevy::{prelude::*, utils::HashMap};

use once_cell::sync::Lazy;
use serde::Deserialize;

use self::slime::{slime_run_system, slime_turn_around_system, GreenSlime};
use crate::{
    asset_loaders::{cache::AssetCache, EmbeddedAssets},
    POST_COLLISION_STAGE,
};

pub static ENEMY_MAP: Lazy<HashMap<EnemyKind, EnemyDescriptor>> = Lazy::new(|| {
    let mut map = HashMap::new();
    map.insert(EnemyKind::GreenSlime, GreenSlime::enemy_descriptor());

    map
});

pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(slime_run_system)
            .add_system(enemy_die_system)
            .add_system_to_stage(POST_COLLISION_STAGE, slime_turn_around_system);
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Hash, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum EnemyKind {
    GreenSlime,
}

pub trait Enemy: Component {
    fn spawn_at(
        commands: &mut Commands,
        image_assets: &mut Assets<Image>,
        asset_cache: &mut AssetCache<EmbeddedAssets>,
        at: Vec3,
    ) -> Entity;
}

pub trait EnemyExt: Enemy {
    fn enemy_descriptor() -> EnemyDescriptor {
        EnemyDescriptor {
            name: std::any::type_name::<Self>(),
            spawn_at: Self::spawn_at,
        }
    }
}

impl<T: Enemy> EnemyExt for T {}

pub struct EnemyDescriptor {
    pub name: &'static str,
    pub spawn_at:
        fn(&mut Commands, &mut Assets<Image>, &mut AssetCache<EmbeddedAssets>, Vec3) -> Entity,
}

impl EnemyDescriptor {
    pub fn spawn_at(
        &self,
        commands: &mut Commands,
        image_assets: &mut Assets<Image>,
        asset_cache: &mut AssetCache<EmbeddedAssets>,
        at: Vec3,
    ) -> Entity {
        (self.spawn_at)(commands, image_assets, asset_cache, at)
    }
}

impl Debug for EnemyDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "EnemyDescriptor({})", self.name)
    }
}

#[derive(Component)]
pub struct EnemyHealth(u32);

impl EnemyHealth {
    pub fn new(start_health: u32) -> Self {
        Self(start_health)
    }
}

impl Sub<u32> for EnemyHealth {
    type Output = Self;

    fn sub(self, rhs: u32) -> Self::Output {
        Self(self.0.saturating_sub(rhs))
    }
}

impl SubAssign<u32> for EnemyHealth {
    fn sub_assign(&mut self, rhs: u32) {
        self.0 = self.0.saturating_sub(rhs);
    }
}

pub fn enemy_die_system(mut commands: Commands, query: Query<(&EnemyHealth, Entity)>) {
    for (enemy, entity) in query.iter() {
        if enemy.0 == 0 {
            commands.entity(entity).despawn();
        }
    }
}
