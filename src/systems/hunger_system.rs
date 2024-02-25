use crate::{
    components::{HungerClock, HungerState, SufferDamage},
    game_log::GameLog,
    RunState,
};
use specs::prelude::*;

pub struct HungerSystem {}

pub const WELL_FED_NUTRITION: i32 = 650;
const NORMAL_NUTRITION: i32 = 600;
const HUNGRY_NUTRITION: i32 = 400;
const STARVING_NUTRITION: i32 = 200;

impl<'a> System<'a> for HungerSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        Entities<'a>,
        WriteStorage<'a, HungerClock>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        WriteStorage<'a, SufferDamage>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, mut hunger_clock, player_entity, run_state, mut inflict_damage, mut log) =
            data;

        for (entity, clock) in (&entities, &mut hunger_clock).join() {
            let mut proceed = false;

            match *run_state {
                RunState::PlayerTurn => {
                    if entity == *player_entity {
                        proceed = true;
                    }
                }
                RunState::MonsterTurn => {
                    if entity != *player_entity {
                        proceed = true;
                    }
                }
                _ => proceed = false,
            }

            if proceed {
                clock.total_nutrition -= 1;

                if clock.total_nutrition > WELL_FED_NUTRITION {
                    clock.total_nutrition = WELL_FED_NUTRITION;
                }

                if in_range(clock.total_nutrition, NORMAL_NUTRITION, WELL_FED_NUTRITION)
                    && clock.state != HungerState::WellFed
                {
                    clock.state = HungerState::WellFed;
                    if entity == *player_entity {
                        log.entries.push("You are well fed!".to_string());
                    }
                } else if in_range(clock.total_nutrition, HUNGRY_NUTRITION, NORMAL_NUTRITION)
                    && clock.state != HungerState::Normal
                {
                    clock.state = HungerState::Normal;
                    if entity == *player_entity {
                        log.entries.push("You are not hungry.".to_string());
                    }
                } else if in_range(clock.total_nutrition, STARVING_NUTRITION, HUNGRY_NUTRITION)
                    && clock.state != HungerState::Hungry
                {
                    clock.state = HungerState::Hungry;
                    if entity == *player_entity {
                        log.entries.push("You are hungry.".to_string());
                        log.entries
                            .push(format!("nutrition: {}", clock.total_nutrition));
                    }
                } else if in_range(clock.total_nutrition, -10000, STARVING_NUTRITION)
                    && clock.state != HungerState::Starving
                {
                    clock.state = HungerState::Starving;
                    if entity == *player_entity {
                        log.entries
                            .push("You are starving! Eat something!".to_string());
                    }
                }

                if clock.total_nutrition < STARVING_NUTRITION {
                    if entity == *player_entity {
                        log.entries.push(
                            "Your hunger pangs are getting painful! You suffer 1 damage."
                                .to_string(),
                        );
                    }
                    SufferDamage::new_damage(&mut inflict_damage, entity, 1);
                }
            }
        }
    }
}

fn in_range(value: i32, min: i32, max: i32) -> bool {
    value >= min && value < max
}
