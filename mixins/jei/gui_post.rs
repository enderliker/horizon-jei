// The recipe screen, over everything the GUI drew (open screens included).
fn jei_gui_post(game: &crate::game::Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    crate::game::jei_glue::gui_render_post(game, batch);
}
