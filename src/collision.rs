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
    // NOTE this is a fix to make the collisions fell more smooth
    const COLLISION_TOLERANCE: f32 = 0.0001;

    for (mut moving_transform, moving_collider, entity) in movable_collider_query.iter_mut() {
        for (static_transform, static_collider) in collider_query.iter() {
            let collision = collide_aabb::collide(
                moving_transform.translation,
                moving_collider.size + Vec2::new(COLLISION_TOLERANCE, COLLISION_TOLERANCE),
                // moving_collider.size,
                static_transform.translation,
                static_collider.size,
            );

            if let Some(collision) = collision {
                println!(
                    "Coll: {:?}, static_pos: {}, moving_pos: {}",
                    collision, static_transform.translation, moving_transform.translation
                );
                if let Some(trans) = next_out_of_bounds_translation(
                    &collision,
                    &moving_transform,
                    moving_collider,
                    static_transform,
                    static_collider,
                ) {
                    println!(
                        "Pos is: {}, Goal pos is: {}",
                        moving_transform.translation, trans
                    );
                    if (trans - moving_transform.translation).abs().length() > COLLISION_TOLERANCE {
                        println!("Moving!");
                        moving_transform.translation = trans;
                        collision_events_writer.send(CollisionEvent { collision, entity });
                    } else {
                        println!("INSIDE!");
                    }
                }
            }
        }
    }
}

fn next_out_of_bounds_translation(
    collision: &Collision,
    moving_transform: &Transform,
    moving_collider: &MoveableCollider,
    static_transform: &Transform,
    static_collider: &Collider,
) -> Option<Vec3> {
    match collision {
        Collision::Top | Collision::Bottom => {
            let signum = collision_signum(collision);
            Some(Vec3::new(
                moving_transform.translation.x,
                static_transform.translation.y
                    + signum * static_collider.size.y / 2.0
                    + signum * moving_collider.size.y / 2.0,
                moving_transform.translation.z,
            ))
        }
        Collision::Right | Collision::Left => {
            let signum = collision_signum(collision);
            Some(Vec3::new(
                static_transform.translation.x
                    + signum * static_collider.size.x / 2.0
                    + signum * moving_collider.size.x / 2.0,
                moving_transform.translation.y,
                moving_transform.translation.z,
            ))
        }
        Collision::Inside => None,
    }
}

fn collision_signum(collision: &Collision) -> f32 {
    match collision {
        Collision::Left | Collision::Bottom => -1.0,
        Collision::Top | Collision::Right => 1.0,
        Collision::Inside => 0.0,
    }
}

#[derive(Component, Debug)]
pub struct BreakableCollider;
