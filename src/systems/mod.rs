use specs::prelude::*;

pub mod damage_system;
pub mod duration_system;
pub mod hunger_system;
pub mod inventory_system;
pub mod map_indexing_system;
pub mod melee_combat_system;
pub mod monster_ai_system;
pub mod particle_system;
pub mod saveload_system;
pub mod status_effects_system;
pub mod trigger_system;
pub mod visibility_system;

pub fn run_systems(ecs: &mut World) {
    let mut visibility = visibility_system::VisibilitySystem {};
    visibility.run_now(ecs);

    let mut monster_ai = monster_ai_system::MonsterAI {};
    monster_ai.run_now(ecs);

    let mut triggers = trigger_system::TriggerSystem {};
    triggers.run_now(ecs);

    let mut map_indexing = map_indexing_system::MapIndexingSystem {};
    map_indexing.run_now(ecs);

    let mut melee_combat = melee_combat_system::MeleeCombatSystem {};
    melee_combat.run_now(ecs);

    let mut damage = damage_system::DamageSystem {};
    damage.run_now(ecs);

    let mut item_collection = inventory_system::ItemCollectionSystem {};
    item_collection.run_now(ecs);

    let mut item_drop = inventory_system::ItemDropSystem {};
    item_drop.run_now(ecs);

    let mut use_items = inventory_system::ItemUseSystem {};
    use_items.run_now(ecs);

    let mut remove_items = inventory_system::ItemRemoveSystem {};
    remove_items.run_now(ecs);

    let mut duration_system = duration_system::DurationSystem {};
    duration_system.run_now(ecs);

    let mut status_effects = status_effects_system::StatusEffectsSystem {};
    status_effects.run_now(ecs);

    let mut status_effects = particle_system::ParticleSpawnSystem {};
    status_effects.run_now(ecs);

    let mut hunger = hunger_system::HungerSystem {};
    hunger.run_now(ecs);

    ecs.maintain();
}
