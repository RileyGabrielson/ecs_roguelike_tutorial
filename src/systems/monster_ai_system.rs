use crate::{components, systems::particle_system, Map, RunState};
use rltk::Point;
use specs::prelude::*;

pub struct MonsterAI {}

impl<'a> System<'a> for MonsterAI {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteExpect<'a, Map>,
        ReadExpect<'a, Point>,
        ReadExpect<'a, Entity>,
        ReadExpect<'a, RunState>,
        Entities<'a>,
        WriteStorage<'a, components::Viewshed>,
        WriteStorage<'a, components::Monster>,
        WriteStorage<'a, components::Position>,
        WriteStorage<'a, components::WantsToMelee>,
        ReadStorage<'a, components::Confusion>,
        ReadStorage<'a, components::Invisible>,
        WriteExpect<'a, particle_system::ParticleBuilder>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            mut map,
            player_pos,
            player_entity,
            run_state,
            entities,
            mut viewshed,
            mut monster,
            mut position,
            mut wants_to_melee,
            confused,
            invisible,
            mut particle_builder,
        ) = data;

        if *run_state != RunState::MonsterTurn {
            return;
        };

        for (entity, viewshed, monster, pos) in
            (&entities, &mut viewshed, &mut monster, &mut position).join()
        {
            let mut can_act = true;

            let is_player_invisible = invisible.get(*player_entity);
            if let Some(_) = is_player_invisible {
                can_act = false;
                stop_target_player(&mut particle_builder, monster, pos.x, pos.y);
            }

            let is_confused = confused.get(entity);
            if let Some(_) = is_confused {
                can_act = false;
            }

            if can_act {
                let distance =
                    rltk::DistanceAlg::Pythagoras.distance2d(Point::new(pos.x, pos.y), *player_pos);
                if distance < 1.5 {
                    wants_to_melee
                        .insert(
                            entity,
                            components::WantsToMelee {
                                target: *player_entity,
                            },
                        )
                        .expect("Unable to insert attack");
                    try_target_player(&mut particle_builder, monster, pos.x, pos.y);
                } else if viewshed.visible_tiles.contains(&*player_pos) {
                    // Path to the player
                    let path = rltk::a_star_search(
                        map.xy_idx(pos.x, pos.y),
                        map.xy_idx(player_pos.x, player_pos.y),
                        &mut *map,
                    );

                    if path.success && path.steps.len() > 1 {
                        let mut idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = false;
                        pos.x = path.steps[1] as i32 % map.width;
                        pos.y = path.steps[1] as i32 / map.width;
                        idx = map.xy_idx(pos.x, pos.y);
                        map.blocked[idx] = true;
                        viewshed.dirty = true;
                    }
                    try_target_player(&mut particle_builder, monster, pos.x, pos.y);
                } else {
                    stop_target_player(&mut particle_builder, monster, pos.x, pos.y);
                }
            }
        }
    }
}

fn try_target_player(
    particle_builder: &mut particle_system::ParticleBuilder,
    monster: &mut components::Monster,
    x: i32,
    y: i32,
) {
    if !monster.is_targeting_player {
        monster.is_targeting_player = true;
        particle_builder.request(
            x,
            y - 1,
            rltk::RGB::named(rltk::YELLOW),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('!'),
            400.0,
        )
    }
}

fn stop_target_player(
    particle_builder: &mut particle_system::ParticleBuilder,
    monster: &mut components::Monster,
    x: i32,
    y: i32,
) {
    if monster.is_targeting_player {
        monster.is_targeting_player = false;
        particle_builder.request(
            x,
            y - 1,
            rltk::RGB::named(rltk::GREY),
            rltk::RGB::named(rltk::BLACK),
            rltk::to_cp437('?'),
            400.0,
        )
    }
}
