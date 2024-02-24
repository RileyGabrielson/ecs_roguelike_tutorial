use super::{components, Rect, MAP_WIDTH};
use rltk::{RandomNumberGenerator, RGB};
use specs::prelude::*;
use specs::saveload::{MarkedBuilder, SimpleMarker};

const MAX_MONSTERS: i32 = 4;
const MAX_ITEMS: i32 = 2;

const ITEM_LAYER: i32 = 5;
const CHARACTER_LAYER: i32 = 4;

pub fn spawn_room(ecs: &mut World, room: &Rect) {
    let mut monster_spawn_points: Vec<usize> = Vec::new();
    let mut item_spawn_points: Vec<usize> = Vec::new();

    // Scope to keep the borrow checker happy
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        let num_monsters = rng.roll_dice(1, MAX_MONSTERS + 2) - 3;
        let num_items = rng.roll_dice(1, MAX_ITEMS + 2) - 3;

        for _i in 0..num_monsters {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * (MAP_WIDTH as usize)) + x;
                if !monster_spawn_points.contains(&idx) {
                    monster_spawn_points.push(idx);
                    added = true;
                }
            }
        }

        for _i in 0..num_items {
            let mut added = false;
            while !added {
                let x = (room.x1 + rng.roll_dice(1, i32::abs(room.x2 - room.x1))) as usize;
                let y = (room.y1 + rng.roll_dice(1, i32::abs(room.y2 - room.y1))) as usize;
                let idx = (y * (MAP_WIDTH as usize)) + x;
                if !item_spawn_points.contains(&idx) {
                    item_spawn_points.push(idx);
                    added = true;
                }
            }
        }
    }

    // Actually spawn the monsters
    for idx in monster_spawn_points.iter() {
        let x = *idx % (MAP_WIDTH as usize);
        let y = *idx / (MAP_WIDTH as usize);
        random_monster(ecs, x as i32, y as i32);
    }

    // Actually spawn the potions
    for idx in item_spawn_points.iter() {
        let x = *idx % (MAP_WIDTH as usize);
        let y = *idx / (MAP_WIDTH as usize);
        random_item(ecs, x as i32, y as i32);
    }
}

/// Spawns the player and returns his/her entity object.
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

/// Spawns a random monster at a given location
pub fn random_monster(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 2);
    }
    match roll {
        1 => orc(ecs, x, y),
        _ => goblin(ecs, x, y),
    }
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
            range: 8,
            dirty: true,
        })
        .with(components::Monster {})
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

fn random_item(ecs: &mut World, x: i32, y: i32) {
    let roll: i32;
    {
        let mut rng = ecs.write_resource::<RandomNumberGenerator>();
        roll = rng.roll_dice(1, 4);
    }
    match roll {
        1 => health_potion(ecs, x, y),
        2 => magic_missile_scroll(ecs, x, y),
        3 => confusion_scroll(ecs, x, y),
        _ => fireball_scroll(ecs, x, y),
    }
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
