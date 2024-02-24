use crate::{components, random_table::RandomTable, Rect, MAP_WIDTH};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};
use std::collections::HashMap;

const MAX_ROOM_SPAWNS: i32 = 4;
const ITEM_LAYER: i32 = 5;
const CHARACTER_LAYER: i32 = 4;

pub fn spawn_room(ecs: &mut World, room: &Rect, map_depth: i32) {
    let spawn_table = room_table(map_depth);
    let mut spawn_points: HashMap<usize, String> = HashMap::new();

    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_spawns = rng.roll_dice(1, MAX_ROOM_SPAWNS + 3) + (map_depth - 1) - 3;

        for _i in 0..num_spawns {
            let mut added = false;
            let mut tries = 0;
            while !added && tries < 20 {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * MAP_WIDTH as usize) + x;
                if !spawn_points.contains_key(&idx) {
                    spawn_points.insert(idx, spawn_table.roll(&mut rng));
                    added = true;
                } else {
                    tries += 1;
                }
            }
        }
    }

    // Actually spawn the things
    for spawn in spawn_points.iter() {
        let x = (*spawn.0 % MAP_WIDTH as usize) as i32;
        let y = (*spawn.0 / MAP_WIDTH as usize) as i32;

        match spawn.1.as_ref() {
            "Goblin" => goblin(ecs, x, y),
            "Orc" => orc(ecs, x, y),
            "Health Potion" => health_potion(ecs, x, y),
            "Fireball Scroll" => fireball_scroll(ecs, x, y),
            "Confusion Scroll" => confusion_scroll(ecs, x, y),
            "Magic Missile Scroll" => magic_missile_scroll(ecs, x, y),
            "Dagger" => dagger(ecs, x, y),
            "Shield" => shield(ecs, x, y),
            _ => {}
        }
    }
}

pub fn player(ecs: &mut World, player_x: i32, player_y: i32) -> Entity {
    let player = ecs
        .create_entity()
        .with(components::Position {
            x: player_x,
            y: player_y,
        })
        .with(components::Renderable {
            layer: CHARACTER_LAYER,
            glyph: rltk::to_cp437('@'),
            fg: RGB::named(rltk::YELLOW),
            bg: RGB::named(rltk::BLACK),
        })
        .with(components::Player {})
        .with(components::Viewshed {
            visible_tiles: Vec::new(),
            range: 8,
            dirty: true,
        })
        .with(components::Name {
            name: "Player".to_string(),
        })
        .with(components::CombatStats {
            max_hp: 30,
            hp: 30,
            defense: 2,
            power: 5,
        })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();

    player
}

pub fn invisibility_timer(ecs: &mut World) -> Entity {
    ecs.create_entity()
        .with(components::Renderable {
            layer: CHARACTER_LAYER,
            glyph: rltk::to_cp437('v'),
            fg: RGB::named(rltk::SILVER),
            bg: RGB::named(rltk::BLACK),
        })
        .with(components::Name {
            name: "Invisibility Timer".to_string(),
        })
        .with(components::Item {})
        .with(components::AppliesInvisiblity { turns: 18 })
        .with(components::Cooldown { turns: 60 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build()
}

pub fn confusion_wand(ecs: &mut World) -> Entity {
    ecs.create_entity()
        .with(components::Renderable {
            layer: CHARACTER_LAYER,
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
        })
        .with(components::Name {
            name: "Confusion Wand".to_string(),
        })
        .with(components::Item {})
        .with(components::CausesConfusion { turns: 5 })
        .with(components::Ranged { range: 6 })
        .with(components::Cooldown { turns: 45 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build()
}

fn orc(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('o'), "Orc");
}
fn goblin(ecs: &mut World, x: i32, y: i32) {
    monster(ecs, x, y, rltk::to_cp437('g'), "Goblin");
}

fn monster<S: ToString>(ecs: &mut World, x: i32, y: i32, glyph: rltk::FontCharType, name: S) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            layer: CHARACTER_LAYER,
            glyph,
            fg: RGB::named(rltk::RED),
            bg: RGB::named(rltk::BLACK),
        })
        .with(components::Viewshed {
            visible_tiles: Vec::new(),
            range: 5,
            dirty: true,
        })
        .with(components::Monster {
            is_targeting_player: false,
        })
        .with(components::Name {
            name: name.to_string(),
        })
        .with(components::BlocksTile {})
        .with(components::CombatStats {
            max_hp: 16,
            hp: 16,
            defense: 1,
            power: 4,
        })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn health_potion(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            layer: ITEM_LAYER,
            glyph: rltk::to_cp437('¡'),
            fg: RGB::named(rltk::MAGENTA),
            bg: RGB::named(rltk::BLACK),
        })
        .with(components::Name {
            name: "Health Potion".to_string(),
        })
        .with(components::Item {})
        .with(components::Consumable {})
        .with(components::ProvidesHealing { heal_amount: 8 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn magic_missile_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            layer: ITEM_LAYER,
        })
        .with(components::Name {
            name: "Magic Missile Scroll".to_string(),
        })
        .with(components::Item {})
        .with(components::Consumable {})
        .with(components::Ranged { range: 6 })
        .with(components::InflictsDamage { damage: 8 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn fireball_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::ORANGE),
            bg: RGB::named(rltk::BLACK),
            layer: ITEM_LAYER,
        })
        .with(components::Name {
            name: "Fireball Scroll".to_string(),
        })
        .with(components::Item {})
        .with(components::Consumable {})
        .with(components::Ranged { range: 6 })
        .with(components::InflictsDamage { damage: 20 })
        .with(components::AreaOfEffect { radius: 3 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn confusion_scroll(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            glyph: rltk::to_cp437(')'),
            fg: RGB::named(rltk::PINK),
            bg: RGB::named(rltk::BLACK),
            layer: ITEM_LAYER,
        })
        .with(components::Name {
            name: "Confusion Scroll".to_string(),
        })
        .with(components::Item {})
        .with(components::Consumable {})
        .with(components::Ranged { range: 6 })
        .with(components::CausesConfusion { turns: 4 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn dagger(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            glyph: rltk::to_cp437('/'),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            layer: ITEM_LAYER,
        })
        .with(components::Name {
            name: "Dagger".to_string(),
        })
        .with(components::Item {})
        .with(components::Equippable {
            slot: components::EquipmentSlot::Melee,
        })
        .with(components::MeleePowerBonus { power: 2 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn shield(ecs: &mut World, x: i32, y: i32) {
    ecs.create_entity()
        .with(components::Position { x, y })
        .with(components::Renderable {
            glyph: rltk::to_cp437('('),
            fg: RGB::named(rltk::CYAN),
            bg: RGB::named(rltk::BLACK),
            layer: ITEM_LAYER,
        })
        .with(components::Name {
            name: "Shield".to_string(),
        })
        .with(components::Item {})
        .with(components::Equippable {
            slot: components::EquipmentSlot::Shield,
        })
        .with(components::DefenseBonus { defense: 1 })
        .marked::<SimpleMarker<components::SerializeMe>>()
        .build();
}

fn room_table(map_depth: i32) -> RandomTable {
    RandomTable::new()
        .add("Goblin", 10)
        .add("Orc", 1 + map_depth)
        .add("Health Potion", 7)
        .add("Fireball Scroll", 2 + map_depth)
        .add("Confusion Scroll", 2 + map_depth)
        .add("Magic Missile Scroll", 4)
        .add("Dagger", 3)
        .add("Shield", 3)
}
