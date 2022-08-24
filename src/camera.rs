use bevy::prelude::*;

/// Be followed by all cameras that have the `FollowEntity` component
#[derive(Component, Debug)]
pub struct FollowedByCamera;

/// For cameras that should follow the entity with the `FollowEntity` component
#[derive(Component, Debug)]
pub struct FollowEntity;

pub fn camera_follow_system(
    mut camera_query: Query<&mut Transform, (With<Camera>, With<FollowEntity>)>,
    followed_by_camera_query: Query<
        &Transform,
        (
            With<FollowedByCamera>,
            Without<Camera>,
            Without<FollowEntity>,
        ),
    >,
) {
    for mut camera_transform in camera_query.iter_mut() {
        match followed_by_camera_query.get_single() {
            Ok(entity_transform) => camera_transform.translation = entity_transform.translation,
            Err(e) => panic!(
                "There is not exactly one entity with FollowedByCamera component: {}",
                e
            ),
        }
    }
}
