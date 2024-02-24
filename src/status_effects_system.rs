use super::components;
use specs::prelude::*;

pub struct StatusEffectsSystem {}

impl<'a> System<'a> for StatusEffectsSystem {
    #[allow(clippy::type_complexity)]
    type SystemData = (
        WriteStorage<'a, components::WantsBeInvisible>,
        WriteStorage<'a, components::Invisible>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut wants_invisibles, mut invisibles) = data;

        for wants_invisible in (&wants_invisibles).join() {
            invisibles
                .insert(
                    wants_invisible.entity,
                    components::Invisible {
                        turns: wants_invisible.turns,
                    },
                )
                .expect("Failed to insert invisiblity");
        }

        wants_invisibles.clear();
    }
}
