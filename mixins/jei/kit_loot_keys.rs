// The ids of the loot tables, for JEI's sources.
use super::LootTables;

impl LootTables {
    /// Every loot table's id, for the kit's sources.
    #[allow(dead_code)]
    pub fn hz_kit_ids_jei(&self) -> Vec<String> {
        self.0.keys().cloned().collect()
    }
}
