// The wheel over the recipe screen turns its pages.
fn jei_mouse_scroll(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    scroll: [f64; 2],
    modifiers: u32,
    canceled: &mut bool,
) {
    crate::game::jei_glue::screen_mouse_scrolled(game, screen, scroll, modifiers, canceled);
}
