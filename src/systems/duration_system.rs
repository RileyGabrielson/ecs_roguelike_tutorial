use crate::{components, RunState};
use specs::prelude::*;

pub struct DurationSystem {}

impl<'a> System<'a> for DurationSystem {
    type SystemData = (
        Entities<'a>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, components::Invisible>,
        WriteStorage<'a, components::ActiveCooldown>,
        WriteStorage<'a, components::Confusion>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, run_state, mut invisibles, mut active_cooldowns, mut confusion) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        }

        let mut entities_to_remove: Vec<Entity> = vec![];
        {
            for (entity, invisible) in (&entities, &mut invisibles).join() {
                invisible.turns -= 1;
                if invisible.turns < 1 {
                    entities_to_remove.push(entity);
                }
            }
        }
        for entity in entities_to_remove {
            invisibles.remove(entity);
        }

        let mut entities_to_remove: Vec<Entity> = vec![];
        {
            for (entity, active_cooldown) in (&entities, &mut active_cooldowns).join() {
                active_cooldown.turns_remaining -= 1;
                if active_cooldown.turns_remaining < 1 {
                    entities_to_remove.push(entity);
                }
            }
        }
        for entity in entities_to_remove {
            active_cooldowns.remove(entity);
        }

        let mut entities_to_remove: Vec<Entity> = vec![];
        {
            for (entity, confused) in (&entities, &mut confusion).join() {
                confused.turns -= 1;
                if confused.turns < 1 {
                    entities_to_remove.push(entity);
                }
            }
        }
        for entity in entities_to_remove {
            confusion.remove(entity);
        }
    }
}
