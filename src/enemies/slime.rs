use bevy::{prelude::*, sprite::collide_aabb::Collision};

use crate::{
    asset_loaders::{cache::AssetCache, EmbeddedAssets},
    collision::{Collider, CollisionEvent, CollisionFilter, MoveableCollider},
    map::TILE_SIZE,
    physics::{Gravity, VelocityId, VelocityMap},
};

use super::{Enemy, EnemyHealth};

const SLIME_BASE_HEALTH: u32 = 8;

#[derive(Component)]
pub struct GreenSlime {
    walking_left: bool,
    vel_id: VelocityId,
}

impl GreenSlime {
    fn new(vel_map: &mut VelocityMap) -> Self {
        Self {
            walking_left: true,
            vel_id: vel_map.register().0,
        }
    }
}

impl Enemy for GreenSlime {
    fn spawn_at(
        commands: &mut Commands,
        image_assets: &mut Assets<Image>,
        asset_cache: &mut AssetCache<EmbeddedAssets>,
        at: Vec3,
    ) -> Entity {
        let texture = asset_cache
            .load_image(image_assets, "sprites/enemies/slime.png")
            .unwrap_or_else(|_| panic!("Failed to load Green slimes texture"));

        let mut vel_map = VelocityMap::new();
        commands
            .spawn_bundle(SpriteBundle {
                texture,
                transform: Transform::from_translation(at + Vec3::new(0.0, TILE_SIZE / 2.0, 0.0)),
                ..Default::default()
            })
            .insert(GreenSlime::new(&mut vel_map))
            .insert(Gravity::new_in(&mut vel_map))
            .insert(Collider {
                filter: CollisionFilter::ALL,
                size: Vec2::new(8.0, 16.0),
                collision_offset: Vec2::ZERO,
            })
            .insert(MoveableCollider)
            .insert(vel_map)
            .insert(EnemyHealth::new(SLIME_BASE_HEALTH))
            .id()
    }
}

pub fn slime_run_system(mut query: Query<(&GreenSlime, &mut Transform, &mut VelocityMap)>) {
    const SLIME_SPEED: f32 = 1.0;
    for (slime, mut trans, mut vel_map) in query.iter_mut() {
        if let Some(vel) = vel_map.get_mut(slime.vel_id) {
            if slime.walking_left {
                vel.x = -SLIME_SPEED;
                trans.rotation = Quat::from_axis_angle(Vec3::Y, 0.0);
            } else {
                vel.x = SLIME_SPEED;
                trans.rotation = Quat::from_axis_angle(Vec3::Y, 180.0f32.to_radians());
            }
        } else {
            panic!("Slime without velocity id");
        }
    }
}

pub fn slime_turn_around_system(
    mut query: Query<&mut GreenSlime>,
    mut collision_reader: EventReader<CollisionEvent>,
) {
    for event in collision_reader.iter() {
        if let Ok(mut slime) = query.get_mut(event.moving_entity) {
            match event.collision {
                Collision::Right => slime.walking_left = false,
                Collision::Left => slime.walking_left = true,
                _ => (),
            }
        }
    }
}
