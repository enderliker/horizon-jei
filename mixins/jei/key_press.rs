// A key over an open screen: the recipe screen's Escape, Backspace and the
// viewer's keys over its stacks.
fn jei_key_press(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    key: &str,
    repeat: bool,
    modifiers: u32,
    canceled: &mut bool,
) {
    if crate::game::jei_glue::is_overlay_open() {
        crate::game::jei_glue::screen_key_pressed(game, screen, key, repeat, modifiers, canceled);
        return;
    }
    if crate::game::hz_kit_window_jei::is_open() {
        crate::game::hz_kit_window_jei::key(game, [game.cursor.0, game.cursor.1], key, canceled);
        if *canceled {
            return;
        }
    }
    crate::game::jei_glue::screen_key_pressed(game, screen, key, repeat, modifiers, canceled);
}
