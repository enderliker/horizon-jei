// A release on a container screen: a click on a bookmark that never left its
// slot asks for the stack's recipes, opened on the next frame.
fn jei_mouse_release(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    if crate::game::jei_glue::is_overlay_open() {
        crate::game::jei_glue::screen_mouse_released(game, screen, button, modifiers, canceled);
        return;
    }
    if crate::game::hz_kit_window_jei::is_open() {
        crate::game::hz_kit_window_jei::release([game.cursor.0, game.cursor.1]);
    }
    crate::game::jei_glue::screen_mouse_released(game, screen, button, modifiers, canceled);
}
