// A screen closes: the recipe screen over it goes with it.
fn jei_screen_closing(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    if crate::game::hz_kit_window_jei::is_open() {
        crate::game::hz_kit_window_jei::close();
    }
    crate::game::jei_glue::screen_closing(game, screen);
}
