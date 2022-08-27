//! Simple implementation of collisions

use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::physics::VelocityMap;

#[derive(Debug)]
pub struct CollisionEvent {
    pub collision: Collision,
    pub moving_entity: Entity,
    pub static_entity: Entity,
}

impl CollisionEvent {
    pub fn new(collision: Collision, moving_entity: Entity, static_entity: Entity) -> Self {
        Self {
            collision,
            moving_entity,
            static_entity,
        }
    }
}

/// A static non moving collider
#[derive(Component, Debug)]
pub struct Collider {
    pub size: Vec2,
}

/// A movable collider which should not pass through
#[derive(Component, Debug)]
pub struct MoveableCollider {
    /// The actual size of the collider
    pub size: Vec2,
    /// The offset used for collision "delay",
    ///
    /// If for instance collision from top is measured, the finite line used for intersection
    /// is extended by `collision_offset.x` to the left AND the right
    pub collision_offset: Vec2,
}

#[allow(clippy::too_many_lines)] // NOTE may be changed later
pub fn collision_system(
    // mut meshes: ResMut<Assets<Mesh>>,
    mut moving_query: Query<(&mut Transform, &VelocityMap, &MoveableCollider, Entity)>,
    collider_query: Query<(&Transform, &Collider, Entity), Without<MoveableCollider>>,
    mut wcollision_events: EventWriter<CollisionEvent>,
) {
    // Check line intersection
    for (mut moving_trans, velocity_map, moving_collider, moving_entity) in moving_query.iter_mut()
    {
        // player_move points
        let start_pos = moving_trans.translation.truncate() - velocity_map.last_velocity();
        let end_pos = moving_trans.translation.truncate();
        let mut next_pos = end_pos;
        // Distance from edges of the moving collider to center
        let moving_coll_offset = moving_collider.size / 2.0;

        for (collider_trans, collider, static_entity) in collider_query.iter() {
            // rect points
            // rect center
            let coll_center = collider_trans.translation.truncate();
            // rect corners
            let bottom_left = coll_center - collider.size / 2.0;
            let top_left = Vec2::new(bottom_left.x, bottom_left.y + collider.size.y);
            let top_right = top_left + Vec2::new(collider.size.x, 0.0);
            let bottom_right = bottom_left + Vec2::new(collider.size.x, 0.0);

            let collision_offset = moving_collider.collision_offset;

            // TODO maybe extract some more (be careful PERFORMANCE!)
            if coll_center.x > start_pos.x
                && top_left.x >= start_pos.x
                && top_left.x <= end_pos.x + moving_coll_offset.x
            {
                let collision_right = line_intersection(
                    start_pos,
                    Vec2::new(end_pos.x + moving_coll_offset.x, end_pos.y),
                    top_left,
                    bottom_left,
                );

                if let Some(intersection) = collision_right {
                    if intersection.y >= bottom_right.y - collision_offset.y
                        && intersection.y <= top_right.y + collision_offset.y
                        && ((intersection.y >= start_pos.y
                            && intersection.y <= end_pos.y + collision_offset.y)
                            || (intersection.y <= start_pos.y
                                && intersection.y >= end_pos.y - collision_offset.y))
                    {
                        wcollision_events.send(CollisionEvent::new(
                            Collision::Left,
                            moving_entity,
                            static_entity,
                        ));
                        let p = Vec2::new(
                            intersection.x - moving_coll_offset.x,
                            moving_trans.translation.y,
                        );
                        if (p.x - start_pos.x).abs() < (next_pos.x - start_pos.x).abs() {
                            next_pos.x = p.x;
                        }
                    }
                }
            }

            if coll_center.y < start_pos.y
                && top_left.y <= start_pos.y
                && top_left.y >= end_pos.y - moving_coll_offset.y
            {
                let collision_top = line_intersection(
                    start_pos,
                    Vec2::new(end_pos.x, end_pos.y - moving_coll_offset.y),
                    top_left,
                    top_right,
                );

                if let Some(intersection) = collision_top {
                    if intersection.x >= top_left.x - collision_offset.x
                        && intersection.x <= top_right.x + collision_offset.x
                        && ((intersection.x >= start_pos.x
                            && intersection.x <= end_pos.x + collision_offset.x)
                            || (intersection.x <= start_pos.x
                                && intersection.x >= end_pos.x - collision_offset.x))
                    {
                        wcollision_events.send(CollisionEvent::new(
                            Collision::Top,
                            moving_entity,
                            static_entity,
                        ));
                        let p = Vec2::new(
                            moving_trans.translation.x,
                            intersection.y + moving_coll_offset.y,
                        );
                        if (p.y - start_pos.y).abs() < (next_pos.y - start_pos.y).abs() {
                            next_pos.y = p.y;
                        }
                    }
                }
            }

            if let Some(coll_right) = check_collide_right(
                coll_center,
                start_pos,
                end_pos,
                moving_coll_offset,
                bottom_right,
                top_right,
                collision_offset,
            ) {
                wcollision_events.send(CollisionEvent::new(
                    Collision::Right,
                    moving_entity,
                    static_entity,
                ));

                if (coll_right - start_pos.x).abs() < (next_pos.x - start_pos.x).abs() {
                    next_pos.x = coll_right;
                }
            }

            if coll_center.y > start_pos.y
                && top_left.y >= start_pos.y
                && top_left.y <= end_pos.y + moving_coll_offset.y * 2.0
            {
                let collision_bottom = line_intersection(
                    start_pos,
                    Vec2::new(end_pos.x, end_pos.y + moving_coll_offset.y),
                    bottom_left,
                    bottom_right,
                );

                if let Some(intersection) = collision_bottom {
                    if intersection.x >= top_left.x - collision_offset.x
                        && intersection.x <= top_right.x + collision_offset.x
                        && ((intersection.x >= start_pos.x
                            && intersection.x <= end_pos.x + collision_offset.x)
                            || (intersection.x <= start_pos.x
                                && intersection.x >= end_pos.x - collision_offset.x))
                    {
                        wcollision_events.send(CollisionEvent::new(
                            Collision::Bottom,
                            moving_entity,
                            static_entity,
                        ));
                        let p = Vec2::new(
                            moving_trans.translation.x,
                            intersection.y - moving_coll_offset.y,
                        );
                        if (p.y - start_pos.y).abs() < (next_pos.y - start_pos.y).abs() {
                            next_pos.y = p.y;
                        }
                    }
                }
            }
        }
        moving_trans.translation = next_pos.extend(0.0);
    }
}

fn line_intersection(
    start_pos: Vec2,
    end_pos: Vec2,
    coll_start: Vec2,
    coll_end: Vec2,
) -> Option<Vec2> {
    let a0 = end_pos.y - start_pos.y;
    let b0 = start_pos.x - end_pos.x;
    let c0 = a0 * start_pos.x + b0 * start_pos.y;

    let a1 = coll_start.y - coll_end.y;
    let b1 = coll_end.x - coll_start.x;
    let c1 = a1 * coll_end.x + b1 * coll_end.y;

    // Calculate determinant
    let determinant = a0 * b1 - a1 * b0;
    if determinant == 0.0 {
        None
    } else {
        let intersection = Vec2::new(
            (b1 * c0 - b0 * c1) / determinant,
            (a0 * c1 - a1 * c0) / determinant,
        );

        Some(intersection)
    }
}

fn check_collide_right(
    coll_center: Vec2,
    start_pos: Vec2,
    end_pos: Vec2,
    moving_coll_offset: Vec2,
    bottom_right: Vec2,
    top_right: Vec2,
    collision_offset: Vec2,
) -> Option<f32> {
    if coll_center.x < start_pos.x
        && bottom_right.x <= start_pos.x
        && bottom_right.x >= end_pos.x - moving_coll_offset.x
    {
        let collision_right = line_intersection(
            start_pos,
            Vec2::new(end_pos.x - moving_coll_offset.x, end_pos.y),
            top_right,
            bottom_right,
        );

        if let Some(intersection) = collision_right {
            if intersection.y >= bottom_right.y - collision_offset.y
                && intersection.y <= top_right.y + collision_offset.y
                && ((intersection.y >= start_pos.y
                    && intersection.y <= end_pos.y + collision_offset.y)
                    || (intersection.y <= start_pos.y
                        && intersection.y >= end_pos.y - collision_offset.y))
            {
                return Some(intersection.x + moving_coll_offset.x);
            }
        }
    }

    None
}

#[derive(Component, Debug)]
pub struct BreakableCollider;
