// The panel's events: the search box, R/U over a stack, Back.
fn jei_event(
    game: &mut crate::game::Game<'_>,
    mod_id: &str,
    event: &crate::game::native_hooks::NativePanelEvent,
) {
    if mod_id != crate::game::jei::ID {
        return;
    }
    crate::game::jei_glue::event(game, event);
    crate::game::hz_kit_layer_jei::event(game, mod_id, event);
}
