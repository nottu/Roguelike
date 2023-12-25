use specs::prelude::*;

use crate::components::ToDelete;

pub struct CleanUpSystem;

impl<'a> System<'a> for CleanUpSystem {
    type SystemData = (Entities<'a>, ReadStorage<'a, ToDelete>);
    fn run(&mut self, data: Self::SystemData) {
        let (entities, to_delete) = data;
        for (entity, _) in (&entities, &to_delete).join() {
            entities.delete(entity).expect("Failed to cleanup entity");
        }
    }
}
