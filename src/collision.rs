//! Simple implementation of collisions

use bevy::{
    prelude::*,
    sprite::collide_aabb::{self, Collision},
};

#[derive(Debug)]
pub struct CollisionEvent {
    pub collision: Collision,
    pub entity: Entity,
}

/// A static non moving collider
#[derive(Component, Debug)]
pub struct Collider {
    pub size: Vec2,
}

/// A movable collider which should not pass through
#[derive(Component, Debug)]
pub struct MoveableCollider {
    pub size: Vec2,
}

/// Check for collisions between movable and static colliders.
/// The movable colliders should be moved outside the static colliders bounds
pub fn collision_system(
    collider_query: Query<(&Transform, &Collider)>,
    mut movable_collider_query: Query<
        (&mut Transform, &MoveableCollider, Entity),
        Without<Collider>,
    >,
    mut collision_events_writer: EventWriter<CollisionEvent>,
) {
    for (mut moving_transform, moving_collider, entity) in movable_collider_query.iter_mut() {
        for (static_transform, static_collider) in collider_query.iter() {
            let collision = collide_aabb::collide(
                moving_transform.translation,
                moving_collider.size,
                static_transform.translation,
                static_collider.size,
            );

            if let Some(collision) = collision {
                move_out_of_bounds(
                    &collision,
                    &mut *moving_transform,
                    moving_collider,
                    static_transform,
                    static_collider,
                );
                collision_events_writer.send(CollisionEvent { collision, entity });
            }
        }
    }
}

fn move_out_of_bounds(
    collision: &Collision,
    moving_transform: &mut Transform,
    moving_collider: &MoveableCollider,
    static_transform: &Transform,
    static_collider: &Collider,
) {
    match collision {
        Collision::Top | Collision::Bottom => {
            let signum = collision_signum(collision);
            moving_transform.translation = Vec3::new(
                // tr1.translation.x + col1.size.x,
                moving_transform.translation.x,
                static_transform.translation.y
                    + signum * static_collider.size.y / 2.0
                    + signum * moving_collider.size.y / 2.0,
                moving_transform.translation.z,
            );
        }
        Collision::Right | Collision::Left => {
            let signum = collision_signum(collision);
            moving_transform.translation = Vec3::new(
                static_transform.translation.x
                    + signum * static_collider.size.x / 2.0
                    + signum * moving_collider.size.x / 2.0,
                moving_transform.translation.y,
                moving_transform.translation.z,
            );
        }
        Collision::Inside => (),
    }
}

fn collision_signum(collision: &Collision) -> f32 {
    match collision {
        Collision::Left | Collision::Bottom => -1.0,
        Collision::Top | Collision::Right => 1.0,
        Collision::Inside => 0.0,
    }
}
