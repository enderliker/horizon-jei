// A screen closes: the recipe screen over it goes with it.
fn jei_screen_closing(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
) {
    crate::game::jei_glue::screen_closing(game, screen);
}
