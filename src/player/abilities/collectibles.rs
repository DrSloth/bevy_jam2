use bevy::{prelude::*, sprite::collide_aabb};

use crate::collision::MoveableCollider;

use super::{Ability, AbilityDescriptor, PlayerInventory};

#[derive(Component, Debug)]
pub struct CollectibleAbilityTrigger {
    pub size: Vec2,
    pub offset: Vec3,
    ability: AbilityDescriptor,
}

impl CollectibleAbilityTrigger {
    pub fn new<T: Ability>(size: Vec2, offset: Vec3) -> Self {
        Self {
            size,
            offset,
            ability: T::descriptor(),
        }
    }
}

/// TODO create its own system for "triggering" the player

pub fn collect_ability_system(
    mut commands: Commands,
    trigger_query: Query<(&CollectibleAbilityTrigger, &Transform, Entity)>,
    mut player_query: Query<(
        &Transform,
        &MoveableCollider,
        &mut Sprite,
        &mut PlayerInventory,
        Entity,
    )>,
    key_events: ResMut<Input<KeyCode>>,
) {
    for (trigger, trigger_transform, trigger_entity) in trigger_query.iter() {
        for (player_transform, player_collider, mut player_sprite, mut inventory, player_entity) in
            player_query.iter_mut()
        {
            let collision = collide_aabb::collide(
                player_transform.translation,
                player_collider.size,
                trigger_transform.translation + trigger.offset,
                trigger.size,
            );

            if let Some(_col) = collision {
                player_sprite.color = Color::LIME_GREEN;

                for key in key_events.get_just_pressed() {
                    if key == &KeyCode::W {
                        commands.entity(trigger_entity).despawn();
                        if let Some(equip_slot) = inventory.first_free_slot() {
                            trigger.ability.equip(
                                &mut commands.entity(player_entity),
                                &mut inventory,
                                equip_slot,
                            );
                        }
                        player_sprite.color = Color::RED;
                    }
                }
            } else {
                player_sprite.color = Color::RED;
            }
        }
    }
}
