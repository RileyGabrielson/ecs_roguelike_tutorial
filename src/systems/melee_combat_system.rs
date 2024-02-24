use crate::{components, GameLog};
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
                let target_stats = combat_stats.get(wants_melee.target).unwrap();
                if target_stats.hp > 0 {
                    let target_name = names.get(wants_melee.target).unwrap();

                    let damage = i32::max(0, stats.power - target_stats.defense);

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
