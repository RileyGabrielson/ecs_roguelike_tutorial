use crate::{components, systems::particle_system, GameLog, Map};
use specs::prelude::*;

pub struct ItemCollectionSystem {}

impl<'a> System<'a> for ItemCollectionSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        WriteStorage<'a, components::WantsToPickupItem>,
        WriteStorage<'a, components::Position>,
        ReadStorage<'a, components::Name>,
        WriteStorage<'a, components::InInventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (player_entity, mut gamelog, mut wants_pickup, mut positions, names, mut backpack) =
            data;

        for pickup in wants_pickup.join() {
            positions.remove(pickup.item);
            backpack
                .insert(
                    pickup.item,
                    components::InInventory {
                        owner: pickup.collected_by,
                    },
                )
                .expect("Unable to insert backpack entry");

            if pickup.collected_by == *player_entity {
                gamelog.entries.push(format!(
                    "You pick up the {}.",
                    names.get(pickup.item).unwrap().name
                ));
            }
        }

        wants_pickup.clear();
    }
}

pub struct ItemUseSystem {}

impl<'a> System<'a> for ItemUseSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        ReadExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, components::WantsToUseItem>,
        ReadStorage<'a, components::Name>,
        ReadStorage<'a, components::Consumable>,
        ReadStorage<'a, components::ProvidesHealing>,
        ReadStorage<'a, components::InflictsDamage>,
        WriteStorage<'a, components::CombatStats>,
        WriteStorage<'a, components::SufferDamage>,
        ReadStorage<'a, components::AreaOfEffect>,
        ReadStorage<'a, components::CausesConfusion>,
        WriteStorage<'a, components::Confusion>,
        ReadStorage<'a, components::AppliesInvisiblity>,
        WriteStorage<'a, components::WantsBeInvisible>,
        ReadStorage<'a, components::Cooldown>,
        WriteStorage<'a, components::ActiveCooldown>,
        WriteExpect<'a, particle_system::ParticleBuilder>,
        ReadStorage<'a, components::Position>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            map,
            entities,
            mut wants_use,
            names,
            consumables,
            healing,
            inflict_damage,
            mut combat_stats,
            mut suffer_damage,
            aoe,
            causes_confusion,
            mut confused,
            applies_invisibility,
            mut wants_be_invisible,
            cooldowns,
            mut active_cooldowns,
            mut particle_builder,
            positions,
        ) = data;

        for (entity, use_item) in (&entities, &wants_use).join() {
            let mut used_item = true;
            let mut can_use_item = true;

            let item_on_cooldown = active_cooldowns.get(use_item.item);
            match item_on_cooldown {
                None => {}
                Some(cooldown) => {
                    if entity == *player_entity {
                        gamelog.entries.push(format!(
                            "You cannot use {}, it is on cooldown for {} turns",
                            names.get(use_item.item).unwrap().name,
                            cooldown.turns_remaining
                        ));
                    }
                    can_use_item = false;
                    used_item = false;
                }
            }

            if can_use_item {
                // Targeting
                let mut targets: Vec<Entity> = Vec::new();
                match use_item.target {
                    None => {
                        targets.push(*player_entity);
                    }
                    Some(target) => {
                        let area_effect = aoe.get(use_item.item);
                        match area_effect {
                            None => {
                                // Single target in tile
                                let tile_idx = map.xy_idx(target.x, target.y);
                                for mob in map.tile_content[tile_idx].iter() {
                                    targets.push(*mob);
                                }
                            }
                            Some(area_effect) => {
                                // AoE
                                let mut blast_tiles =
                                    rltk::field_of_view(target, area_effect.radius, &*map);
                                blast_tiles.retain(|p| {
                                    p.x > 0
                                        && p.x < map.width - 1
                                        && p.y > 0
                                        && p.y < map.height - 1
                                });
                                for tile_idx in blast_tiles.iter() {
                                    let idx = map.xy_idx(tile_idx.x, tile_idx.y);
                                    for mob in map.tile_content[idx].iter() {
                                        targets.push(*mob);
                                    }
                                    particle_builder.request(
                                        tile_idx.x,
                                        tile_idx.y,
                                        rltk::RGB::named(rltk::ORANGE),
                                        rltk::RGB::named(rltk::BLACK),
                                        rltk::to_cp437('░'),
                                        200.0,
                                    );
                                }
                            }
                        }
                    }
                }

                // If it heals, apply the healing
                let item_heals = healing.get(use_item.item);
                match item_heals {
                    None => {}
                    Some(healer) => {
                        used_item = false;
                        for target in targets.iter() {
                            let stats = combat_stats.get_mut(*target);
                            if let Some(stats) = stats {
                                stats.hp = i32::min(stats.max_hp, stats.hp + healer.heal_amount);
                                if entity == *player_entity {
                                    gamelog.entries.push(format!(
                                        "You use the {}, healing {} hp.",
                                        names.get(use_item.item).unwrap().name,
                                        healer.heal_amount
                                    ));
                                }
                                used_item = true;

                                let pos = positions.get(*target);
                                if let Some(pos) = pos {
                                    particle_builder.request(
                                        pos.x,
                                        pos.y,
                                        rltk::RGB::named(rltk::GREEN),
                                        rltk::RGB::named(rltk::BLACK),
                                        rltk::to_cp437('♥'),
                                        200.0,
                                    );
                                }
                            }
                        }
                    }
                }

                // If it inflicts damage, apply it to the target cell
                let item_damages = inflict_damage.get(use_item.item);
                match item_damages {
                    None => {}
                    Some(damage) => {
                        used_item = false;
                        for mob in targets.iter() {
                            components::SufferDamage::new_damage(
                                &mut suffer_damage,
                                *mob,
                                damage.damage,
                            );
                            if entity == *player_entity {
                                let mob_name = names.get(*mob).unwrap();
                                let item_name = names.get(use_item.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {} on {}, inflicting {} hp.",
                                    item_name.name, mob_name.name, damage.damage
                                ));
                            }

                            used_item = true;
                            let pos = positions.get(*mob);
                            if let Some(pos) = pos {
                                particle_builder.request(
                                    pos.x,
                                    pos.y,
                                    rltk::RGB::named(rltk::RED),
                                    rltk::RGB::named(rltk::BLACK),
                                    rltk::to_cp437('‼'),
                                    200.0,
                                );
                            }
                        }
                    }
                }

                // If it applies invisiblity, apply it
                let makes_invisible = applies_invisibility.get(use_item.item);
                match makes_invisible {
                    None => {}
                    Some(invisible) => {
                        used_item = false;
                        for mob in targets.iter() {
                            wants_be_invisible
                                .insert(
                                    *mob,
                                    components::WantsBeInvisible {
                                        entity: *mob,
                                        turns: invisible.turns,
                                    },
                                )
                                .expect("Failed to insert wants be invisible");
                            if entity == *player_entity {
                                let item_name = names.get(use_item.item).unwrap();
                                gamelog.entries.push(format!(
                                    "You use {}, and become invisible for {} turns.",
                                    item_name.name, invisible.turns
                                ));
                            }

                            used_item = true;
                        }
                    }
                }

                // Can it pass along confusion?
                let mut add_confusion = Vec::new();
                {
                    let item_causes_confusion = causes_confusion.get(use_item.item);
                    match item_causes_confusion {
                        None => {}
                        Some(confusion) => {
                            used_item = false;
                            for mob in targets.iter() {
                                add_confusion.push((*mob, confusion.turns));
                                if entity == *player_entity {
                                    let mob_name = names.get(*mob).unwrap();
                                    let item_name = names.get(use_item.item).unwrap();
                                    gamelog.entries.push(format!(
                                        "You use {} on {}, confusing them.",
                                        item_name.name, mob_name.name
                                    ));

                                    let pos = positions.get(*mob);
                                    if let Some(pos) = pos {
                                        particle_builder.request(
                                            pos.x,
                                            pos.y,
                                            rltk::RGB::named(rltk::MAGENTA),
                                            rltk::RGB::named(rltk::BLACK),
                                            rltk::to_cp437('?'),
                                            400.0,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
                for (mob_entity, remaining_turns) in add_confusion.iter() {
                    confused
                        .insert(
                            *mob_entity,
                            components::Confusion {
                                turns: *remaining_turns,
                            },
                        )
                        .expect("Unable to insert status");
                }
            }

            if used_item {
                // If its a consumable, we delete it on use
                let consumable = consumables.get(use_item.item);
                match consumable {
                    None => {}
                    Some(_) => {
                        entities.delete(use_item.item).expect("Delete failed");
                    }
                }

                // If it has a cooldown, add an active cooldown
                let has_cooldown = cooldowns.get(use_item.item);
                match has_cooldown {
                    None => {}
                    Some(cooldown) => {
                        active_cooldowns
                            .insert(
                                use_item.item,
                                components::ActiveCooldown {
                                    turns_remaining: cooldown.turns,
                                },
                            )
                            .expect("Failed to insert active cooldown");
                    }
                }
            }
        }

        wants_use.clear();
    }
}

pub struct ItemDropSystem {}

impl<'a> System<'a> for ItemDropSystem {
    type SystemData = (
        ReadExpect<'a, Entity>,
        WriteExpect<'a, GameLog>,
        Entities<'a>,
        WriteStorage<'a, components::WantsToDropItem>,
        ReadStorage<'a, components::Name>,
        WriteStorage<'a, components::Position>,
        WriteStorage<'a, components::InInventory>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            player_entity,
            mut gamelog,
            entities,
            mut wants_drop,
            names,
            mut positions,
            mut backpack,
        ) = data;

        for (entity, to_drop) in (&entities, &wants_drop).join() {
            let mut dropper_pos = components::Position { x: 0, y: 0 };
            {
                let dropped_pos = positions.get(entity).unwrap();
                dropper_pos.x = dropped_pos.x;
                dropper_pos.y = dropped_pos.y;
            }
            positions
                .insert(
                    to_drop.item,
                    components::Position {
                        x: dropper_pos.x,
                        y: dropper_pos.y,
                    },
                )
                .expect("Unable to insert position");
            backpack.remove(to_drop.item);

            if entity == *player_entity {
                gamelog.entries.push(format!(
                    "You drop the {}.",
                    names.get(to_drop.item).unwrap().name
                ));
            }
        }

        wants_drop.clear();
    }
}
