use bevy::{prelude::*, sprite::collide_aabb};

use crate::collision::Collider;

use super::{Ability, AbilityDescriptor, EquipSlot, PlayerInventory};

#[derive(Component, Debug)]
pub struct CollectibleAbilityTrigger {
    pub size: Vec2,
    pub offset: Vec3,
    ability: AbilityDescriptor,
}

impl CollectibleAbilityTrigger {
    #[allow(dead_code)] // Useful for testing
    pub fn new<T: Ability>(size: Vec2, offset: Vec3) -> Self {
        Self::new_with_descriptor(size, offset, T::ability_descriptor())
    }

    pub fn new_with_descriptor(size: Vec2, offset: Vec3, ability: AbilityDescriptor) -> Self {
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
        &mut Sprite,
        &mut PlayerInventory,
        Entity,
    )>,
    key_events: ResMut<Input<KeyCode>>,
) {
    let mut mark = false;

    for (player_transform, player_collider, mut player_sprite, mut inventory, player_entity) in
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
                mark = true;
                for key in key_events.get_just_pressed() {
                    if let Some(equip_slot) = EquipSlot::from_equipkey(*key) {
                        commands.entity(trigger_entity).despawn();
                        mark = false;
                        trigger.ability.equip(
                            &mut commands.entity(player_entity),
                            &mut inventory,
                            equip_slot,
                        );
                    }
                }
            }
        }

        if mark {
            player_sprite.color = Color::GREEN;
        } else {
            player_sprite.color = Color::WHITE;
        }
    }
}
