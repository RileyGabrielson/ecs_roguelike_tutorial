use crate::{components, systems::particle_system, GameLog};
use specs::prelude::*;

pub struct MeleeCombatSystem {}

impl<'a> System<'a> for MeleeCombatSystem {
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, components::WantsToMelee>,
        ReadStorage<'a, components::Name>,
        ReadStorage<'a, components::CombatStats>,
        WriteStorage<'a, components::SufferDamage>,
        WriteStorage<'a, components::Invisible>,
        WriteExpect<'a, GameLog>,
        WriteExpect<'a, particle_system::ParticleBuilder>,
        ReadStorage<'a, components::Position>,
        ReadStorage<'a, components::MeleePowerBonus>,
        ReadStorage<'a, components::DefenseBonus>,
        ReadStorage<'a, components::Equipped>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            mut wants_melee,
            names,
            combat_stats,
            mut inflict_damage,
            mut invisible,
            mut game_log,
            mut particle_builder,
            positions,
            melee_power_bonuses,
            defense_bonuses,
            equipped,
        ) = data;

        for (entity, wants_melee, name, stats) in
            (&entities, &wants_melee, &names, &combat_stats).join()
        {
            let is_invisible = invisible.get(entity);
            if let Some(_) = is_invisible {
                invisible.remove(entity);
                game_log
                    .entries
                    .push(format!("{} attacks, and loses invisibility", &name.name));
            }

            if stats.hp > 0 {
                let mut offensive_bonus = 0;
                for (_item_entity, power_bonus, equipped_by) in
                    (&entities, &melee_power_bonuses, &equipped).join()
                {
                    if equipped_by.owner == entity {
                        offensive_bonus += power_bonus.power;
                    }
                }

                let pos = positions.get(wants_melee.target);
                if let Some(pos) = pos {
                    particle_builder.request(
                        pos.x,
                        pos.y,
                        rltk::RGB::named(rltk::ORANGE),
                        rltk::RGB::named(rltk::BLACK),
                        rltk::to_cp437('‼'),
                        130.0,
                    );
                }
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let mut defensive_bonus = 0;
                    for (_item_entity, defense_bonus, equipped_by) in
                        (&entities, &defense_bonuses, &equipped).join()
                    {
                        if equipped_by.owner == wants_melee.target {
                            defensive_bonus += defense_bonus.defense;
                        }
                    }

                    let damage = i32::max(
                        0,
                        (stats.power + offensive_bonus) - (target_stats.defense + defensive_bonus),
                    );

                    if damage == 0 {
                        game_log.entries.push(format!(
                            "{} is unable to hurt {}",
                            &name.name, &target_name.name
                        ));
                    } else {
                        game_log.entries.push(format!(
                            "{} hits {}, for {} hp.",
                            &name.name, &target_name.name, damage
                        ));
                        components::SufferDamage::new_damage(
                            &mut inflict_damage,
                            wants_melee.target,
                            damage,
                        );
                    }
                }
            }
        }

        wants_melee.clear();
    }
}
