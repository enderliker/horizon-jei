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
    crate::game::jei_glue::screen_key_pressed(game, screen, key, repeat, modifiers, canceled);
}
