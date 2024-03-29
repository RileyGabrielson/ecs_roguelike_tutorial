use crate::{components, game_log::GameLog, Map};
use rltk::{field_of_view, Point};
use specs::prelude::*;

pub struct VisibilitySystem {}

impl<'a> System<'a> for VisibilitySystem {
    type SystemData = (
        WriteExpect<'a, Map>,
        Entities<'a>,
        WriteStorage<'a, components::Viewshed>,
        WriteStorage<'a, components::Position>,
        ReadStorage<'a, components::Player>,
        WriteStorage<'a, components::Invisible>,
        WriteExpect<'a, rltk::RandomNumberGenerator>,
        ReadStorage<'a, components::Name>,
        WriteExpect<'a, GameLog>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            entities,
            mut viewshed,
            pos,
            player,
            mut invisibles,
            mut rng,
            names,
            mut game_log,
        ) = data;

        for (entity, viewshed, position) in (&entities, &mut viewshed, &pos).join() {
            if viewshed.dirty {
                viewshed.dirty = true;
                viewshed.visible_tiles.clear();
                viewshed.visible_tiles =
                    field_of_view(Point::new(position.x, position.y), viewshed.range, &*map);
                viewshed
                    .visible_tiles
                    .retain(|p| p.x >= 0 && p.x < map.width && p.y >= 0 && p.y < map.height);

                // If this is the player, reveal what they can see
                let _p: Option<&components::Player> = player.get(entity);
                if let Some(_p) = _p {
                    for t in map.visible_tiles.iter_mut() {
                        *t = false
                    }
                    for vis in viewshed.visible_tiles.iter() {
                        let idx = map.xy_idx(vis.x, vis.y);
                        map.revealed_tiles[idx] = true;
                        map.visible_tiles[idx] = true;

                        // Chance to reveal hidden things
                        for e in map.tile_content[idx].iter() {
                            if *e != entity {
                                let maybe_hidden = invisibles.get(*e);
                                if let Some(_maybe_hidden) = maybe_hidden {
                                    if rng.roll_dice(1, 24) == 1 {
                                        let name = names.get(*e);
                                        if let Some(name) = name {
                                            game_log
                                                .entries
                                                .push(format!("You spotted a {}.", &name.name));
                                        }
                                        invisibles.remove(*e);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
