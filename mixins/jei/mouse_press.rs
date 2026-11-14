// A press on a container screen: over a bookmark it is the mod's and goes no
// further (it would take a slot's stack or the world's use).
fn jei_mouse_press(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    if crate::game::jei_glue::is_overlay_open() {
        crate::game::jei_glue::screen_mouse_pressed(game, screen, button, modifiers, canceled);
        return;
    }
    if crate::game::hz_kit_window_jei::is_open() {
        crate::game::hz_kit_window_jei::press(
            game,
            [game.cursor.0, game.cursor.1],
            button,
            canceled,
        );
        if *canceled {
            return;
        }
    }
    crate::game::jei_glue::screen_mouse_pressed(game, screen, button, modifiers, canceled);
}
