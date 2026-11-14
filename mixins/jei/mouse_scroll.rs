// The wheel over the recipe screen turns its pages.
fn jei_mouse_scroll(
    game: &crate::game::Game<'_>,
    screen: &crate::game::native_hooks::NativeScreenView<'_>,
    scroll: [f64; 2],
    modifiers: u32,
    canceled: &mut bool,
) {
    if crate::game::jei_glue::is_overlay_open() {
        crate::game::jei_glue::screen_mouse_scrolled(game, screen, scroll, modifiers, canceled);
        return;
    }
    if crate::game::hz_kit_window_jei::is_open() {
        crate::game::hz_kit_window_jei::scroll(scroll[1], canceled);
        if *canceled {
            return;
        }
    }
    crate::game::jei_glue::screen_mouse_scrolled(game, screen, scroll, modifiers, canceled);
}
