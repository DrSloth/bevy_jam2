use bevy::prelude::*;

use crate::map::{MapManager, TILE_SIZE};

/// Be followed by all cameras that have the `FollowEntity` component
#[derive(Component, Debug)]
pub struct FollowedByCamera;

/// For cameras that should follow the entity with the `FollowEntity` component
#[derive(Component, Debug)]
pub struct FollowEntity;

pub fn camera_follow_system(
    mut camera_query: Query<(&mut Transform, &Camera), With<FollowEntity>>,
    followed_by_camera_query: Query<
        &Transform,
        (
            With<FollowedByCamera>,
            Without<Camera>,
            Without<FollowEntity>,
        ),
    >,
    map: Res<MapManager>,
) {
    for (mut camera_transform, camera) in camera_query.iter_mut() {
        match followed_by_camera_query.get_single() {
            Ok(entity_transform) => {
                let room_size = map.room_stack.last().unwrap().size;
                
                let cam_size = camera.logical_target_size().unwrap() * 0.175;
                // let cam_size = camera.logical_target_size().unwrap();
                // let cam_size = Vec2::new(TILE_SIZE * 32.0, TILE_SIZE * 18.0);
                println!("{:?} {:?}", cam_size, camera_transform.translation);
                // return;
                camera_transform.translation.x = entity_transform.translation.x;
                if camera_transform.translation.x - cam_size.x / 2.0 < 0.0 {
                    camera_transform.translation.x = cam_size.x / 2.0;
                }

                if camera_transform.translation.x + cam_size.x / 2.0 > room_size.x {
                    camera_transform.translation.x = room_size.x - cam_size.x / 2.0;
                }

                camera_transform.translation.y = entity_transform.translation.y;
                if camera_transform.translation.y - cam_size.y / 2.0 < 0.0 {
                    camera_transform.translation.y = cam_size.y / 2.0;
                }

                if camera_transform.translation.y + cam_size.y / 2.0 > room_size.y {
                    camera_transform.translation.y = room_size.y - cam_size.y / 2.0;
                }
            }
            Err(e) => panic!(
                "There is not exactly one entity with FollowedByCamera component: {}",
                e
            ),
        }
    }
}
