// The server is about to send its data: ask for the recipe types the viewer
// shows, so a data pack's recipes reach it (and the "+") as the server has them.
fn jei_datapack_sync(game: &mut crate::game::Game<'_>, recipe_types: &mut Vec<String>) {
    if !game.native_on(crate::game::jei::ID) {
        return;
    }
    for kind in crate::game::jei::SYNCED_RECIPE_TYPES {
        if !recipe_types.iter().any(|t| t == kind) {
            recipe_types.push((*kind).to_owned());
        }
    }
}
