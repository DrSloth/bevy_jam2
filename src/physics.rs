use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::collision::{CollisionEvent, MoveableCollider};

pub const GRAVITY: f32 = 0.07;

pub const GRAVITY_MAX: f32 = -9.0;

#[derive(Debug)]
pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(gravity_system)
            .add_system_to_stage(CoreStage::PostUpdate, landing_system)
            .add_system_to_stage(CoreStage::Last, velocity_system);
    }
}

/// An id to a velocity inside a velocity map
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Clone, Copy)]
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

    pub fn set(&mut self, id: VelocityId, vel: Vec2) -> Result<Vec2, VelocityError> {
        if let Some(v) = self.get_mut(id) {
            let old_val = *v;
            *v = vel;
            Ok(old_val)
        } else {
            Err(VelocityError::NotFound)
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum VelocityError {
    #[error("The id is not inside the velocity map, try to register first")]
    NotFound,
}

pub fn velocity_system(mut query: Query<(&mut Transform, &mut VelocityMap)>) {
    for (mut transform, mut velocity_map) in query.iter_mut() {
        let velocity: Vec2 = velocity_map.map.iter().sum();
        let z = transform.translation.z;
        transform.translation += velocity.extend(z);
        velocity_map.last_velocity = velocity;
    }
}

/// Gravity component to make things fall
#[derive(Component, Debug)]
pub struct Gravity {
    pub(crate) vel_id: VelocityId,
}

impl Gravity {
    /// Create a new Gravity without a velocity id
    pub fn new(vel_id: VelocityId) -> Self {
        Self { vel_id }
    }

    pub fn new_in(vel_map: &mut VelocityMap) -> Self {
        Self::new(vel_map.register().0)
    }
}

/// System to apply gravity to all entities with the Gravity components
pub fn gravity_system(mut query: Query<(&mut VelocityMap, &mut Gravity)>) {
    for (mut velocity_map, grav) in query.iter_mut() {
        // transform.translation.y -= GRAVITY;
        let vel = if let Some(vel) = velocity_map.get_mut(grav.vel_id) {
            vel
        } else {
            unreachable!("Requested velocity id doesn't exist");
        };

        vel.y = (vel.y - GRAVITY).max(GRAVITY_MAX);
    }
}

pub fn landing_system(
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut query: Query<(&mut Gravity, &mut VelocityMap), With<MoveableCollider>>,
) {
    for evt in collision_event_reader.iter() {
        match evt.collision {
            Collision::Top => (),
            _ => continue,
        }

        if let Ok((grav, mut velocity_map)) = query.get_mut(evt.entity) {
            if let Some(vel) = velocity_map.get_mut(grav.vel_id) {
                *vel = Vec2::ZERO;
            }
        }
    }
}
