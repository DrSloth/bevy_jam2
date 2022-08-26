//! Simple implementation of collisions

use std::sync::atomic::{self, AtomicUsize};

use bevy::{
    prelude::*,
    sprite::{
        collide_aabb::{self, Collision},
        Mesh2dHandle,
    },
};

use crate::player::MouseCursor;

#[derive(Debug)]
pub struct CollisionEvent {
    pub collision: Collision,
    pub entity: Entity,
}

impl CollisionEvent {
    pub fn new(collision: Collision, entity: Entity) -> Self {
        Self {
            collision,
            entity,
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
    pub size: Vec2,
}

pub fn collision_system(
    mut meshes: ResMut<Assets<Mesh>>,
    line_start_query: Query<&Transform, With<LineStart>>,
    mouse_query: Query<(&Transform, Entity), (With<MouseCursor>, Without<LineStart>)>,
    mut line_query: Query<
        (&mut Transform, &Mesh2dHandle),
        (Without<LineStart>, Without<MouseCursor>, With<Line>),
    >,
    collider_query: Query<
        (&Transform, &Collider),
        (Without<Line>, Without<MouseCursor>, Without<LineStart>),
    >,
    mut wcollision_events: EventWriter<CollisionEvent>,
) {
    // draw line
    for line_start_trans in line_start_query.iter() {
        for (mouse_cursor_trans, _entity) in mouse_query.iter() {
            for (mut line_trans, line_mesh) in line_query.iter_mut() {
                let dist = line_start_trans.translation - mouse_cursor_trans.translation;

                line_trans.translation = line_start_trans.translation - dist / 2.0;
                line_trans.rotation = Quat::from_axis_angle(Vec3::NEG_Z, (dist.x / dist.y).atan());
                let _ = meshes.set(
                    line_mesh.0.clone(),
                    Mesh::from(shape::Quad {
                        size: Vec2::new(1.0, dist.length()),
                        ..Default::default()
                    }),
                );
            }
        }
    }

    static CNT: AtomicUsize = AtomicUsize::new(0);
    // Check line intersection
    for (mouse_cursor_trans, moving_entity) in mouse_query.iter() {
        for line_start_trans in line_start_query.iter() {
            for (collider_trans, collider) in collider_query.iter() {
                // line points
                let start_pos = line_start_trans.translation.truncate();
                let end_pos = mouse_cursor_trans.translation.truncate();
                // rect points
                // rect center
                let coll_center = collider_trans.translation.truncate();
                let bottom_left = coll_center - collider.size / 2.0;
                let top_left = Vec2::new(bottom_left.x, bottom_left.y + collider.size.y);
                let top_right = top_left + Vec2::new(collider.size.x, 0.0);
                let bottom_right = bottom_left + Vec2::new(collider.size.x, 0.0);

                // TODO maybe extract some more (be careful PERFORMANCE!)
                if top_left.x >= start_pos.x && top_left.x <= end_pos.x {
                    let collision_left =
                        line_intersection(start_pos, end_pos, top_left, bottom_left);

                    if let Some(intersection) = collision_left {
                        if intersection.y <= top_left.y && intersection.y >= bottom_left.y {
                            wcollision_events
                                .send(CollisionEvent::new(Collision::Left, moving_entity))
                        }
                    }
                }

                if top_left.y <= start_pos.y && top_left.y >= end_pos.y {
                    let collision_top = line_intersection(start_pos, end_pos, top_left, top_right);

                    if let Some(intersection) = collision_top {
                        if intersection.x >= top_left.x && intersection.x <= top_right.x {
                            let x = CNT.fetch_add(1, atomic::Ordering::Relaxed);
                            println!("Coll top at {}! {}", intersection, x);
                        }
                    }
                }

                if bottom_right.x <= start_pos.x && bottom_right.x >= end_pos.x {
                    let collision_right =
                        line_intersection(start_pos, end_pos, top_right, bottom_right);

                    if let Some(intersection) = collision_right {
                        if intersection.y <= top_right.y && intersection.y >= bottom_right.y {
                            let x = CNT.fetch_add(1, atomic::Ordering::Relaxed);
                            println!("Coll right at {}! {}", intersection, x);
                        }
                    }
                }

                if bottom_right.y >= start_pos.y && bottom_right.y <= end_pos.y {
                    let collision_bottom =
                        line_intersection(start_pos, end_pos, bottom_right, bottom_left);

                    if let Some(intersection) = collision_bottom {
                        if intersection.x <= bottom_right.x && intersection.x >= bottom_left.x {
                            let x = CNT.fetch_add(1, atomic::Ordering::Relaxed);
                            println!("Coll bottom at {}! {}", intersection, x);
                        }
                    }
                }
            }
        }
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
    if determinant != 0.0 {
        let intersection = Vec2::new(
            (b1 * c0 - b0 * c1) / determinant,
            (a0 * c1 - a1 * c0) / determinant,
        );

        Some(intersection)
    } else {
        None
    }
}

#[derive(Component, Debug)]
pub struct BreakableCollider;

#[derive(Component, Debug)]
pub struct LineStart;

#[derive(Component, Debug)]
pub struct Line;
