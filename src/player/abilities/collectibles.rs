use bevy::{prelude::*, sprite::collide_aabb};

use crate::{
    asset_loaders::{cache::AssetCache, EmbeddedAssets},
    collision::Collider,
    map::TILE_SIZE,
    player::PlayerTrigger,
};

use super::{AbilityDescriptor, AbilityId, EquipSlot, PlayerInventory};

#[derive(Component, Debug)]
pub struct CollectibleAbilityTrigger {
    pub size: Vec2,
    pub offset: Vec3,
    ability: &'static AbilityDescriptor,
}

impl CollectibleAbilityTrigger {
    // #[allow(dead_code)] // Useful for testing
    // pub fn new<T: Ability>(size: Vec2, offset: Vec3) -> Self {
    //     Self::new_with_descriptor(size, offset, T::ability_descriptor())
    // }

    pub fn default_with_descriptor(descriptor: &'static AbilityDescriptor) -> Self {
        Self::new_with_descriptor(Vec2::new(32.0, 16.0), Vec3::ZERO, descriptor)
    }

    pub fn new_with_descriptor(
        size: Vec2,
        offset: Vec3,
        ability: &'static AbilityDescriptor,
    ) -> Self {
        Self {
            size,
            offset,
            ability,
        }
    }
}

pub fn collect_ability_system(
    mut commands: Commands,
    trigger_query: Query<(&CollectibleAbilityTrigger, &Transform, Entity)>,
    mut player_query: Query<(
        &Transform,
        &Collider,
        &mut PlayerTrigger,
        &mut PlayerInventory,
        Entity,
    )>,
    key_events: ResMut<Input<KeyCode>>,
) {
    for (player_transform, player_collider, mut player_trigger, mut inventory, player_entity) in
        player_query.iter_mut()
    {
        for (trigger, trigger_transform, trigger_entity) in trigger_query.iter() {
            let collision = collide_aabb::collide(
                player_transform.translation,
                player_collider.size,
                trigger_transform.translation + trigger.offset,
                trigger.size,
            );

            if let Some(_col) = collision {
                player_trigger.trigger_collect();
                for key in key_events.get_just_pressed() {
                    if let Some(equip_slot) = EquipSlot::from_equipkey(*key) {
                        commands.entity(trigger_entity).despawn();
                        trigger.ability.equip(
                            &mut commands.entity(player_entity),
                            &mut inventory,
                            equip_slot,
                        );
                    }
                }
            }
        }
    }
}

#[derive(Component, Debug)]
pub struct CombineAltar {
    pub size: Vec2,
    pub offset: Vec3,
}

pub fn combine_altar_system(
    mut commands: Commands,
    altar_query: Query<(&CombineAltar, &Transform)>,
    mut player_query: Query<(&Collider, &Transform, &mut PlayerTrigger, &PlayerInventory)>,
    key_events: ResMut<Input<KeyCode>>,
    mut image_assets: ResMut<Assets<Image>>,
    mut asset_cache: ResMut<AssetCache<EmbeddedAssets>>,
) {
    for (player_collider, player_transform, mut player_trigger, inv) in player_query.iter_mut() {
        for (altar, altar_transform) in altar_query.iter() {
            let collision = collide_aabb::collide(
                player_transform.translation,
                player_collider.size,
                altar_transform.translation + altar.offset,
                altar.size,
            );

            if collision.is_some() {
                player_trigger.trigger_interact();
                if key_events.just_pressed(KeyCode::W) {
                    let new_item = inv.0.combine(inv.1);
                    println!("{}", new_item.name());
                    if new_item == AbilityId::None {
                        continue;
                    }
                    commands
                        .spawn_bundle(SpriteBundle {
                            texture: asset_cache
                                .load_image(&mut image_assets, new_item.sprite_path())
                                .unwrap_or_else(|e| {
                                    panic!("Failed to load image for {:?}: {}", new_item, e)
                                }),
                            transform: Transform::from_translation(
                                altar_transform.translation
                                    + Vec3::new(TILE_SIZE * 0.5, TILE_SIZE * 4.0, 0.0),
                            ),
                            ..Default::default()
                        })
                        .insert(CollectibleAbilityTrigger::default_with_descriptor(
                            new_item.ability_descriptor(),
                        ));
                }
            }
        }
    }
}
