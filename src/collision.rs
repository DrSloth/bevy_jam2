//! Simple implementation of collisions

use std::{
    cmp::Ordering,
    ops::{BitAnd, BitOr, Not},
};

use bevy::{prelude::*, sprite::collide_aabb::Collision};
use serde::{Deserialize, Serialize};

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
    pub filter: CollisionFilter,
}

impl Default for Collider {
    fn default() -> Self {
        Self {
            size: Vec2::ZERO,
            filter: CollisionFilter::ALL,
        }
    }
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

#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(transparent)]
pub struct CollisionFilter(u8);

impl CollisionFilter {
    pub const TOP: Self = Self(0b1000);
    pub const RIGHT: Self = Self(0b0100);
    pub const BOTTOM: Self = Self(0b0010);
    pub const LEFT: Self = Self(0b0001);
    pub const ALL: Self = Self(0xF);

    // pub fn with(self, other: Self) -> Self {
    //     self | other
    // }

    pub fn collides_at(self, other: CollisionFilter) -> bool {
        (self & other).0 == other.0
    }

    pub fn collides_top(self) -> bool {
        self.collides_at(Self::TOP)
    }

    pub fn collides_right(self) -> bool {
        self.collides_at(Self::RIGHT)
    }

    pub fn collides_bottom(self) -> bool {
        self.collides_at(Self::BOTTOM)
    }

    pub fn collides_left(self) -> bool {
        self.collides_at(Self::LEFT)
    }

    // pub fn collides_all(self) -> bool {
    //     self.collides_at(Self::ALL)
    // }
}

impl BitOr for CollisionFilter {
    type Output = Self;
    fn bitor(self, rhs: Self) -> Self::Output {
        Self(self.0 | rhs.0)
    }
}

impl BitAnd for CollisionFilter {
    type Output = Self;
    fn bitand(self, rhs: Self) -> Self::Output {
        Self(self.0 & rhs.0)
    }
}

impl Not for CollisionFilter {
    type Output = Self;
    fn not(self) -> Self::Output {
        Self(!self.0 & 0xF)
    }
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
        let mut vertical_collisions: Vec<CollisionWith> = vec![];
        let mut horizontal_collisions: Vec<CollisionWith> = vec![];

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
            if collider.filter.collides_left()
                && coll_center.x > start_pos.x
                && top_left.x >= start_pos.x
                && top_left.x <= end_pos.x + moving_coll_offset.x
            {
                let collision_left = line_intersection(
                    start_pos,
                    Vec2::new(end_pos.x + moving_coll_offset.x, end_pos.y),
                    top_left,
                    bottom_left,
                );

                if let Some(intersection) = collision_left {
                    if intersection.y >= bottom_right.y - collision_offset.y
                        && intersection.y <= top_right.y + collision_offset.y
                        && ((intersection.y >= start_pos.y
                            && intersection.y <= end_pos.y + collision_offset.y)
                            || (intersection.y <= start_pos.y
                                && intersection.y >= end_pos.y - collision_offset.y))
                    {
                        let new_x = intersection.x - moving_coll_offset.x;
                        let ord = new_x.total_cmp(&next_pos.x);
                        let collision_with = CollisionWith {
                            static_entity,
                            coll_dir: Collision::Left,
                        };

                        match ord {
                            Ordering::Less => {
                                horizontal_collisions.clear();
                                next_pos.x = new_x;
                                horizontal_collisions.push(collision_with);
                            }
                            Ordering::Equal => {
                                horizontal_collisions.push(collision_with);
                            }
                            Ordering::Greater => (),
                        }
                    }
                }
            }

            if collider.filter.collides_top()
                && top_left.y < start_pos.y
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
                        let new_y = intersection.y + moving_coll_offset.y;
                        let ord = next_pos.y.total_cmp(&new_y);
                        let collision_with = CollisionWith {
                            static_entity,
                            coll_dir: Collision::Top,
                        };

                        match ord {
                            Ordering::Less => {
                                vertical_collisions.clear();
                                next_pos.y = new_y;
                                vertical_collisions.push(collision_with);
                            }
                            Ordering::Equal => {
                                vertical_collisions.push(collision_with);
                            }
                            Ordering::Greater => (),
                        }
                    }
                }
            }

            if collider.filter.collides_right() {
                if let Some(new_x) = check_collide_right(
                    coll_center,
                    start_pos,
                    end_pos,
                    moving_coll_offset,
                    bottom_right,
                    top_right,
                    collision_offset,
                ) {
                    let ord = next_pos.x.total_cmp(&new_x);
                    let collision_with = CollisionWith {
                        static_entity,
                        coll_dir: Collision::Right,
                    };

                    match ord {
                        Ordering::Less => {
                            horizontal_collisions.clear();
                            next_pos.x = new_x;
                            horizontal_collisions.push(collision_with);
                        }
                        Ordering::Equal => {
                            horizontal_collisions.push(collision_with);
                        }
                        Ordering::Greater => (),
                    }
                }
            }

            if collider.filter.collides_bottom()
                && coll_center.y > start_pos.y
                && bottom_left.y >= start_pos.y
                && bottom_left.y <= end_pos.y + moving_coll_offset.y
            {
                let collision_bottom = line_intersection(
                    start_pos,
                    Vec2::new(end_pos.x, end_pos.y + moving_coll_offset.y),
                    bottom_left,
                    bottom_right,
                );

                if let Some(intersection) = collision_bottom {
                    if intersection.x >= bottom_left.x - collision_offset.x
                        && intersection.x <= bottom_right.x + collision_offset.x
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
                        let new_y = intersection.y + moving_coll_offset.y;
                        let ord = new_y.total_cmp(&next_pos.y);
                        let collision_with = CollisionWith {
                            static_entity,
                            coll_dir: Collision::Bottom,
                        };

                        match ord {
                            Ordering::Less => {
                                vertical_collisions.clear();
                                next_pos.y = new_y;
                                vertical_collisions.push(collision_with);
                            }
                            Ordering::Equal => {
                                vertical_collisions.push(collision_with);
                            }
                            Ordering::Greater => (),
                        }
                    }
                }
            }
        }
        moving_trans.translation = next_pos.extend(0.0);

        for collision in vertical_collisions
            .into_iter()
            .chain(horizontal_collisions.into_iter())
        {
            wcollision_events.send(CollisionEvent {
                static_entity: collision.static_entity,
                collision: collision.coll_dir,
                moving_entity,
            })
        }
    }
}

#[derive(Debug)]
struct CollisionWith {
    static_entity: Entity,
    coll_dir: Collision,
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
