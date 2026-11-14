// The server's recipes arrived (or the window went back to its own): the
// viewer is built again from them on the next frame.
fn jei_recipes_received(game: &mut crate::game::Game<'_>) {
    if game.native_on(crate::game::jei::ID) {
        crate::game::jei_glue::recipes_received();
    }
}
