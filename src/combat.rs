//! Structures primarily for combat

use bevy::prelude::*;

use crate::physics::{VelocityId, VelocityMap};

/// A projectile moved along its direction
#[derive(Debug, Component)]
pub struct Projectile {
    pub direction: Vec2,
    pub speed: f32,
    pub(crate) vel_id: Option<VelocityId>,
}

/// System to move a projectile along their direction with their speed
pub fn move_projectile_system(mut projectiles: Query<(&mut VelocityMap, &mut Projectile)>) {
    for (mut vel_map, mut projectile) in projectiles.iter_mut() {
        // transform.translation += (projectile.direction * projectile.speed).extend(0.0);
        let vel = if let Some(vel) = projectile.vel_id.and_then(|id| vel_map.get_mut(id)) {
            vel
        } else {
            let (id, vel) = vel_map.register();
            projectile.vel_id = Some(id);
            vel
        };

        *vel = projectile.direction * projectile.speed;
    }
}
