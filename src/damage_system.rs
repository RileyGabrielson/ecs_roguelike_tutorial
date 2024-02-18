use super::{CombatStats, GameLog, Name, Player, SufferDamage};
use specs::prelude::*;

pub struct DamageSystem {}

impl<'a> System<'a> for DamageSystem {
    type SystemData = (
        WriteStorage<'a, CombatStats>,
        WriteStorage<'a, SufferDamage>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut stats, mut damage) = data;

        for (stats, damage) in (&mut stats, &damage).join() {
            stats.hp -= damage.amount.iter().sum::<i32>();
        }

        damage.clear();
    }
}

pub fn delete_the_dead(ecs: &mut World) {
    let mut dead: Vec<Entity> = Vec::new();
    // Using a scope to make the borrow checker happy
    {
        let combat_stats = ecs.read_storage::<CombatStats>();
        let players = ecs.read_storage::<Player>();
        let names = ecs.read_storage::<Name>();
        let mut log = ecs.write_resource::<GameLog>();
        let entities = ecs.entities();

        for (entity, stats, name) in (&entities, &combat_stats, &names).join() {
            if stats.hp < 1 {
                let player = players.get(entity);
                match player {
                    None => {
                        dead.push(entity);
                        log.entries.push(format!("{} is dead.", name.name))
                    }
                    Some(_) => log.entries.push("You are dead.".to_string()),
                }
            }
        }
    }

    for victim in dead {
        ecs.delete_entity(victim).expect("Unable to delete");
    }
}
