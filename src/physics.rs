use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::collision::{CollisionEvent, MovableCollider};

pub const GRAVITY: f32 = 1.9;
pub const GRAVITY_MAX: f32 = -22.0;

/// An id to a velocity inside a velocity map
#[derive(Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
pub struct VelocityId(usize);

/// A map of velocities set by different component.
///
/// Every component can add its own velocity to this map to be applied after the update stage.
#[derive(Component, Debug, Default, Clone)]
pub struct VelocityMap {
    /// The backing storage of all velocities
    map: Vec<Vec2>,
    last_velocity: Vec2,
}

impl VelocityMap {
    /// Create a new empty velocity map
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new velocity
    pub fn register(&mut self) -> (VelocityId, &mut Vec2) {
        let id = self.map.len();
        self.map.push(Vec2::ZERO);
        let id = VelocityId(id);
        (
            id,
            self.get_mut(id)
                .unwrap_or_else(|| unreachable!("Wrong velocity allocation")),
        )
    }

    pub fn get_mut(&mut self, id: VelocityId) -> Option<&mut Vec2> {
        self.map.get_mut(id.0)
    }

    // pub fn last_velocity(&self) -> Vec2 {
    //     self.last_velocity
    // }

    pub fn get(&self, id: VelocityId) -> Option<Vec2> {
        self.map.get(id.0).copied()
    }

    pub fn set(&mut self, id: VelocityId, vel: Vec2) -> Option<Vec2> {
        if let Some(v) = self.get_mut(id) {
            let old_val = *v;
            *v = vel;
            Some(old_val)
        } else {
            None
        }
    }
}

pub fn velocity_system(mut query: Query<(&mut Transform, &mut VelocityMap)>) {
    for (mut transform, mut velocity_map) in query.iter_mut() {
        let velocity: Vec2 = velocity_map.map.iter().sum();
        transform.translation = Vec3::new(
            transform.translation.x + velocity.x,
            transform.translation.y + velocity.y,
            transform.translation.z,
        );
        velocity_map.last_velocity = velocity;
    }
}

/// Gravity component to make things fall
#[derive(Component, Debug, Default)]
pub struct Gravity {
    vel_id: Option<VelocityId>,
}

impl Gravity {
    /// Create a new Gravity without a velocity id
    pub fn new() -> Self {
        Self::default()
    }

    pub fn vel_id(&self) -> &Option<VelocityId> {
        &self.vel_id
    }
}

/// System to apply gravity to all entities with the Gravity components
pub fn gravity_system(mut query: Query<(&mut VelocityMap, &mut Gravity)>) {
    for (mut velocity_map, mut grav) in query.iter_mut() {
        // transform.translation.y -= GRAVITY;
        let vel = if let Some(vel) = grav.vel_id.and_then(|id| velocity_map.get_mut(id)) {
            vel
        } else {
            let (id, vel) = velocity_map.register();
            grav.vel_id = Some(id);
            vel
        };

        vel.y = (vel.y - GRAVITY).max(GRAVITY_MAX);
    }
}

pub fn landing_system(
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut query: Query<(&mut Gravity, &mut VelocityMap), With<MovableCollider>>,
) {
    for evt in collision_event_reader.iter() {
        match evt.collision {
            Collision::Top => (),
            _ => continue,
        }

        if let Ok((grav, mut velocity_map)) = query.get_mut(evt.entity) {
            if let Some(vel) = grav.vel_id.and_then(|id| velocity_map.get_mut(id)) {
                *vel = Vec2::ZERO;
            }
        }
    }
}
