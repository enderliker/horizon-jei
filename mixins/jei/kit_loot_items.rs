// The loot tables an `Items` holds, for JEI's sources.
use super::Items;

impl Items {
    /// The loaded loot tables, for the kit's sources.
    #[allow(dead_code)]
    pub fn hz_kit_tables_jei(&self) -> std::sync::Arc<horizon_world::loot::LootTables> {
        self.loot.clone()
    }
}
