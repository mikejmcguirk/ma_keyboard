use crate::{population::Population, structs::IdSpawner};

pub struct MetaPopulation {
    id_spawner: IdSpawner,
    collection: Vec<Population>,
}

// TODO: Start preparing for meta
// - Move all structs to holding their own RNG. More memory + managing re-seeding, but better
// positions us for multi-threading + avoids moving references up and down the call stack. For the
// kbs, re-create the kb_copy method with a re-seed, and disable cloning to clean up any clones
// - For logging, log some debug message in setup and remove those calls from Population. I don't
// know what I want to do logging for, so no need to do it speculatively. File accessing is
// touchier than RNG because I think you need to lock it to use it. Multi-threading issue
// - See population for specifics there
// - No user-level edits for population management or meta-population management. All has to be
// correct at compile time
impl MetaPopulation {
    pub fn create() -> Self {
        let mut id_spawner = IdSpawner::new();

        let mut collection = Vec::new();
        for _ in 0..20 {
            let new_pop = Population::create(id_spawner.get());
            collection.push(new_pop);
        }

        return Self {
            id_spawner,
            collection,
        };
    }
}
