// The edges between Horizon and the viewer (crate::game::jei): the game's
// item and recipe tables in, the panel, give and place out, and the overlay
// (tooltips, the bookmark bar and its clicks) wired to the viewer.
use crate::game::Game;
use crate::game::jei::{
    self, Bookmarks, Config, Context, Counts, Event, Ingredient, Item, Kind, Layout, Out, Recipe,
    Session, Sort, Stack, Toggle, View, Viewer, screen as geo,
};
use crate::game::native_hooks::{
    NativePanelEvent, NativeRecipeKind, NativeRecipeLayout, NativeRecipeStack, NativeRecipeView,
    NativeRecipes, NativeScreenView, NativeStack,
};
use horizon_client::items::ItemStack;
use std::cell::RefCell;
use std::path::PathBuf;

/// Variants one item adds to the list (potions, tipped arrows).
const VARIANTS_PER_ITEM: usize = 8;

/// Shift's GLFW bit.
const SHIFT: u32 = 1;

/// The viewer while a world is open, and what the overlay keeps.
struct State {
    session: Session,
    bookmarks: Bookmarks,
    config: Config,
    /// The config and the bookmarks have been read from disk.
    loaded: bool,
    /// Actions a hook that only borrows the game leaves to the frame.
    pending: Vec<Event>,
    /// The bookmark under the left button, while it is down.
    drag: Option<Drag>,
    /// The screen drawn last (`screen_render_post`): its id and menu type,
    /// for the "+" (which grid is open). None once a frame passes without one.
    screen: Option<(&'static str, Option<String>)>,
    /// A screen was drawn since the last frame began.
    screen_seen: bool,
    /// Where this frame's screen is and the pointer, for the recipe screen
    /// `gui_render_post` draws.
    overlay_at: Option<([f32; 4], [f32; 2])>,
    /// The server's recipes arrived (`native_recipes_received`): build the
    /// viewer again from them.
    rebuild: bool,
    /// JEI's recipe screen, while it is open.
    overlay: Option<Overlay>,
    /// The last press went to the recipe screen: its release and drags too.
    overlay_pressed: bool,
    /// The GUI's size at the last frame drawn, for the input hooks' hit tests.
    gui: [f32; 2],
    /// Stacks the overlay's keys bookmark at the next frame.
    to_bookmark: Vec<Stack>,
}

/// The recipe screen: the viewer's page, the page of it shown, and whether
/// JEI opened a screen of its own to show it (nothing was open).
struct Overlay {
    title: String,
    tabs: Vec<Stack>,
    names: Vec<String>,
    tab: usize,
    views: Vec<View>,
    catalysts: Vec<Stack>,
    subtitle: String,
    page: usize,
    own_screen: bool,
}

/// A bookmark being dragged to another slot.
#[derive(Clone, Copy, Debug)]
struct Drag {
    from: usize,
    /// The pointer moved to another slot: a drop reorders instead of opening.
    moved: bool,
}

impl State {
    fn new() -> Self {
        Self {
            session: Session::new(),
            bookmarks: Bookmarks::default(),
            config: Config::default(),
            loaded: false,
            pending: Vec::new(),
            drag: None,
            screen: None,
            screen_seen: false,
            overlay_at: None,
            rebuild: false,
            overlay: None,
            overlay_pressed: false,
            gui: [0.0, 0.0],
            to_bookmark: Vec::new(),
        }
    }

    fn viewer(&self) -> Option<&Viewer> {
        self.session.viewer()
    }

    fn viewer_mut(&mut self) -> Option<&mut Viewer> {
        self.session.viewer_mut()
    }
}

thread_local! {
    static STATE: RefCell<State> = RefCell::new(State::new());
}

/// The folder the mod keeps its files in, `config/jei/`, inside the game's.
fn dir() -> Option<PathBuf> {
    let mut path = Game::native_config_dir().ok()?;
    path.push(jei::ID);
    Some(path)
}

fn config_path() -> Option<PathBuf> {
    Some(dir()?.join("jei.cfg"))
}

fn bookmarks_path() -> Option<PathBuf> {
    Some(dir()?.join("bookmarks.txt"))
}

fn read(path: Option<PathBuf>) -> Option<String> {
    std::fs::read_to_string(path?).ok()
}

fn write(path: Option<PathBuf>, text: &str) {
    let Some(path) = path else {
        return;
    };
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    let _ = std::fs::write(path, text);
}

/// Reads the config and the bookmarks once: `config/jei/jei.cfg` and
/// `config/jei/bookmarks.txt`.
fn load() {
    if STATE.with(|s| s.borrow().loaded) {
        return;
    }
    let config = read(config_path()).map_or_else(Config::default, |text| Config::parse(&text));
    let bookmarks =
        read(bookmarks_path()).map_or_else(Bookmarks::default, |text| jei::bookmarks_parse(&text));
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.config = config;
        state.bookmarks = bookmarks;
        state.loaded = true;
    });
}

/// The config now, read from disk the first time.
pub(super) fn config() -> Config {
    load();
    STATE.with(|s| s.borrow().config)
}

fn store_config(config: Config) {
    STATE.with(|s| s.borrow_mut().config = config);
    write(config_path(), &config.to_text());
}

fn save_bookmarks(bookmarks: &Bookmarks) {
    write(bookmarks_path(), &jei::bookmarks_text(bookmarks));
}

/// Where the config and the bookmarks live, for `/jei config`.
pub(super) fn files() -> String {
    match (config_path(), bookmarks_path()) {
        (Some(config), Some(bookmarks)) => {
            format!(
                "JEI files: {} and {}",
                config.display(),
                bookmarks.display()
            )
        }
        _ => "JEI: the game's config folder is unknown".to_owned(),
    }
}

/// The player's language.
fn lang(game: &Game<'_>) -> jei::Lang {
    jei::lang(game.native_setting(jei::ID, "language").as_deref())
}

/// Builds the viewer once per world and shows the item list in the panel.
pub(super) fn sync(game: &mut Game<'_>) {
    let has_player = game.native_player().is_some();
    let enabled = game.native_on(jei::ID);
    let needs_build = STATE.with(|s| {
        let mut state = s.borrow_mut();
        if !std::mem::take(&mut state.screen_seen) {
            state.screen = None;
        }
        let rebuild = has_player && enabled && std::mem::take(&mut state.rebuild);
        if rebuild {
            state.session.clear();
        }
        let needs_build = state.session.update(has_player, enabled);
        if !has_player {
            state.drag = None;
            state.pending.clear();
        }
        needs_build
    });
    if !needs_build {
        return;
    }
    let Some(catalog) = game.native_recipes() else {
        return;
    };
    load();
    let sort = STATE.with(|s| s.borrow().config.sort);
    let recipes = recipes(&catalog);
    let found = jei::harvest_variants(&recipes, VARIANTS_PER_ITEM, jei::ITEMS_MAX);
    let (mut viewer, mut out) = Viewer::with_variants(items(&catalog), recipes, found, lang(game));
    if sort != viewer.sort() {
        out = viewer.set_sort(sort);
    }
    // A mod menu type with a crafting grid crafts as the crafting table: its
    // block (the item of the same id) is one more crafting catalyst.
    viewer.set_crafters(
        horizon_client::containers::mod_menu_types()
            .iter()
            .filter(|t| t.crafting.is_some())
            .map(|t| t.id.to_owned())
            .collect(),
    );
    STATE.with(|s| s.borrow_mut().session.set_viewer(viewer));
    if let Err(error) = apply(game, out) {
        game.native_chat(&format!("JEI: {error}"));
    }
}

/// Runs what the overlay's hooks left for it (a bookmark click opens the
/// panel), once a frame, where the game can still be changed.
pub(super) fn flush(game: &mut Game<'_>) {
    if !game.native_on(jei::ID) {
        return;
    }
    let (pending, marks) = STATE.with(|s| {
        let mut state = s.borrow_mut();
        (
            std::mem::take(&mut state.pending),
            std::mem::take(&mut state.to_bookmark),
        )
    });
    for stack in marks {
        bookmark(game, stack);
    }
    for event in pending {
        dispatch(game, event);
    }
}

/// The panel's events: search, clicks, keys, tabs, pages and the action button.
pub(super) fn event(game: &mut Game<'_>, event: &NativePanelEvent) {
    if !game.native_on(jei::ID) {
        return;
    }
    let Some(event) = convert_event(event) else {
        return;
    };
    // The bookmark key bookmarks the stack under the pointer, wherever it is:
    // the game says which one it is.
    if let Event::Key { key, stack } = &event {
        let bookmark_key = game.native_mod_key(jei::ID, "bookmark", jei::BOOKMARK_KEY);
        if *key == bookmark_key {
            bookmark(game, stack.clone());
            return;
        }
    }
    dispatch(game, event);
}

/// Bookmarks a stack, or takes its bookmark away, and says so.
fn bookmark(game: &mut Game<'_>, stack: Stack) {
    let lang = lang(game);
    let result = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let result = state.bookmarks.toggle(&stack);
        if result != Toggle::Full {
            save_bookmarks(&state.bookmarks);
        }
        result
    });
    let name = STATE
        .with(|s| s.borrow().viewer().map(|v| v.item_name(&stack.item)))
        .unwrap_or_else(|| stack.item.clone());
    let text = match result {
        Toggle::Added(_) => format!("{}: {name}", lang.bookmark_added()),
        Toggle::Removed(_) => format!("{}: {name}", lang.bookmark_removed()),
        Toggle::Full => lang.bookmarks_full().to_owned(),
    };
    game.native_chat(&text);
}

/// Takes a bookmark away by its slot, keeping the file.
fn remove_bookmark(index: usize) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        if state.bookmarks.remove(index).is_some() {
            save_bookmarks(&state.bookmarks);
        }
        state.drag = None;
    });
}

/// The bookmark bar over a container screen: its slots, the item in each and
/// the name of the one under the pointer, as the game finished drawing.
/// The recipe screen, after the whole GUI: over the screen it opened on
/// (`screen_render_post` said where that is this frame).
pub(super) fn gui_render_post(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch) {
    let at = STATE.with(|s| s.borrow_mut().overlay_at.take());
    let Some((rect, mouse)) = at else {
        return;
    };
    if !game.native_on(jei::ID) {
        return;
    }
    STATE.with(|s| {
        if let Some(overlay) = s.borrow().overlay.as_ref() {
            draw_overlay(game, batch, rect, mouse, overlay);
        }
    });
}

pub(super) fn screen_render_post(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    screen: &NativeScreenView<'_>,
) {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.screen = Some((screen.id, screen.menu.map(str::to_owned)));
        state.screen_seen = true;
        state.gui = [batch.width, batch.height];
    });
    if !game.native_on(jei::ID) {
        return;
    }
    // The recipe screen itself draws in `gui_render_post`, over this screen
    // and whatever a mod drew on it after us; it needs where this one is.
    STATE.with(|s| s.borrow_mut().overlay_at = Some((screen.rect, screen.mouse)));
    if !bar_screen(screen) {
        return;
    }
    load();
    let (config, bookmarks, drag) = STATE.with(|s| {
        let state = s.borrow();
        (state.config, state.bookmarks.clone(), state.drag)
    });
    if !config.bookmarks || bookmarks.is_empty() {
        return;
    }
    let bar = jei::bookmark_bar(batch.height);
    let count = bookmarks.len().min(bar.capacity());
    let hovered = bar.hit(screen.mouse, count);
    // The bar's box first, so the screen shows through the slots.
    game.native_fill(batch, bar.rect(), 0x5000_0000);
    for index in 0..count {
        let rect = bar.slot(index);
        if hovered == Some(index) || drag.is_some_and(|drag| drag.from == index) {
            game.native_fill(batch, rect, 0x50FF_FFFF);
        }
        if let Some(item) = item_stack(&bookmarks.slots()[index]) {
            game.native_item_stack(batch, &item, [rect[0] + 1.0, rect[1] + 1.0], 16.0);
        }
    }
    let Some(index) = hovered else {
        return;
    };
    let slot = &bookmarks.slots()[index];
    let name = STATE
        .with(|s| s.borrow().viewer().map(|v| v.item_name(&slot.item)))
        .unwrap_or_else(|| slot.item.clone());
    draw_tooltip(
        game,
        batch,
        screen.mouse,
        &jei::bookmark_tooltip(&name, lang(game)),
    );
}

/// The screens the bookmark bar shows on: the containers', and any screen
/// the recipe screen is open over.
fn bar_screen(screen: &NativeScreenView<'_>) -> bool {
    jei::overlay_screen(screen.id) || STATE.with(|s| s.borrow().overlay.is_some())
}

/// A small tooltip at the pointer: the overlay's slots are not the game's, so
/// it draws its own.
fn draw_tooltip(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    mouse: [f32; 2],
    lines: &[(String, [u8; 4])],
) {
    if lines.is_empty() {
        return;
    }
    let mut width: f32 = 0.0;
    for (text, _) in lines {
        width = width.max(game.native_text_width(text));
    }
    let height = lines.len() as f32 * 10.0;
    let x = (mouse[0] + 10.0).min(batch.width - width - 6.0).max(2.0);
    let y = (mouse[1] - 12.0)
        .max(2.0)
        .min((batch.height - height - 6.0).max(2.0));
    game.native_fill(
        batch,
        [x - 3.0, y - 3.0, width + 6.0, height + 6.0],
        0xF0_10_00_10,
    );
    for (n, (text, [r, g, b, _])) in lines.iter().enumerate() {
        let rgb = u32::from_be_bytes([0, *r, *g, *b]);
        game.native_text(batch, text, [x, y + n as f32 * 10.0], rgb, false);
    }
}

/// A press over the bar: the left button takes the slot (opening its recipes
/// when the release comes without a move), the right one opens its uses,
/// shift and the middle button take the bookmark away.
pub(super) fn screen_mouse_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) || !bar_screen(screen) {
        return;
    }
    load();
    let (config, bookmarks) = STATE.with(|s| {
        let state = s.borrow();
        (state.config, state.bookmarks.clone())
    });
    let on_bar = config.bookmarks
        && jei::bookmark_slots()
            .hit(screen.mouse, bookmarks.len())
            .is_some();
    if !on_bar {
        overlay_pressed(game, screen, button, modifiers, canceled);
        return;
    }
    // The corner's places do not depend on the screen's height: a hit test
    // uses the whole bar, of which the screen draws as many rows as fit.
    let Some(index) = jei::bookmark_slots().hit(screen.mouse, bookmarks.len()) else {
        return;
    };
    *canceled = true;
    let stack = bookmarks.slots()[index].clone();
    match button {
        // Left: a click opens the recipes, a drag reorders.
        0 if modifiers & SHIFT == 0 => {
            STATE.with(|s| {
                s.borrow_mut().drag = Some(Drag {
                    from: index,
                    moved: false,
                })
            });
        }
        0 => remove_bookmark(index),
        // Right: the uses.
        1 => pending(Event::Open { stack, uses: true }),
        // Middle: take it away.
        2 => remove_bookmark(index),
        _ => {}
    }
}

/// A release over the bar: a left click that never left its slot opens the
/// stack's recipes.
pub(super) fn screen_mouse_released(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    if STATE.with(|s| std::mem::take(&mut s.borrow_mut().overlay_pressed)) {
        *canceled = true;
        return;
    }
    if !bar_screen(screen) || button != 0 {
        return;
    }
    let Some(drag) = STATE.with(|s| s.borrow_mut().drag.take()) else {
        return;
    };
    *canceled = true;
    if drag.moved {
        return;
    }
    let stack = STATE.with(|s| s.borrow().bookmarks.get(drag.from).cloned());
    if let Some(stack) = stack {
        pending(Event::Open { stack, uses: false });
    }
}

/// Dragging a bookmark over another slot moves it there.
pub(super) fn screen_mouse_dragged(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    _delta: [f32; 2],
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    if STATE.with(|s| s.borrow().overlay_pressed) {
        *canceled = true;
        return;
    }
    if !bar_screen(screen) || button != 0 {
        return;
    }
    let Some(mut drag) = STATE.with(|s| s.borrow().drag) else {
        return;
    };
    let count = STATE.with(|s| s.borrow().bookmarks.len());
    let Some(to) = jei::bookmark_slots().hit(screen.mouse, count) else {
        return;
    };
    *canceled = true;
    if to == drag.from {
        return;
    }
    let moved = STATE.with(|s| {
        let mut state = s.borrow_mut();
        let moved = state.bookmarks.move_slot(drag.from, to);
        if moved {
            save_bookmarks(&state.bookmarks);
        }
        moved
    });
    if moved {
        drag.from = to;
        drag.moved = true;
        STATE.with(|s| s.borrow_mut().drag = Some(drag));
    }
}

fn pending(event: Event) {
    STATE.with(|s| s.borrow_mut().pending.push(event));
}

/// Adds the viewer's lines to any item's tooltip: what makes it, what uses it
/// and whether it is bookmarked.
pub(super) fn tooltip(
    game: &Game<'_>,
    stack: &ItemStack,
    lines: &mut Vec<(horizon_client::text::Text, u32)>,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    load();
    let components = crate::game::native_recipes::components(stack);
    let config = STATE.with(|s| s.borrow().config);
    let counts = STATE.with(|s| {
        let state = s.borrow();
        state.viewer().map_or(Counts::default(), |viewer| {
            viewer.counts(&stack.item, &components)
        })
    });
    let marked = STATE.with(|s| {
        s.borrow()
            .bookmarks
            .position(&Stack {
                item: stack.item.to_string(),
                count: 1,
                components: components.clone(),
            })
            .is_some()
    });
    let counts = if config.hints {
        counts
    } else {
        Counts::default()
    };
    let added = jei::tooltip_lines(counts, marked, lang(game));
    if added.is_empty() {
        return;
    }
    if !lines.is_empty() {
        lines.push((horizon_client::text::Text::literal(""), u32::MAX));
    }
    for (text, [r, g, b, a]) in added {
        let color = u32::from_le_bytes([r, g, b, a]);
        lines.push((horizon_client::text::Text::literal(&text), color));
    }
}

/// The settings page's config, as its buttons show it.
pub(super) fn settings() -> Config {
    config()
}

/// The next order, shown in the list at once.
pub(super) fn cycle_sort(game: &mut Game<'_>) {
    set_sort(game, config().sort.next());
}

pub(super) fn set_sort(game: &mut Game<'_>, sort: Sort) {
    let mut config = config();
    config.sort = sort;
    store_config(config);
    let out = STATE.with(|s| s.borrow_mut().viewer_mut().map(|v| v.set_sort(sort)));
    if let Some(out) = out {
        if let Err(error) = apply(game, out) {
            game.native_chat(&format!("JEI: {error}"));
        }
    }
}

pub(super) fn toggle_hints() -> Config {
    let mut config = config();
    config.hints = !config.hints;
    store_config(config);
    config
}

pub(super) fn toggle_bar() -> Config {
    let mut config = config();
    config.bookmarks = !config.bookmarks;
    store_config(config);
    config
}

/// Empties the bar, keeping the file.
pub(super) fn clear_bookmarks() {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.bookmarks.clear();
        state.drag = None;
        save_bookmarks(&state.bookmarks);
    });
}

/// The crafting grid of the screen drawn last: 0 when none takes recipes.
fn open_grid() -> u32 {
    STATE.with(|s| {
        s.borrow().screen.as_ref().map_or(0, |(id, menu)| {
            let mod_grid = menu
                .as_deref()
                .and_then(horizon_client::containers::mod_menu_type)
                .and_then(|t| t.crafting)
                .map(|c| c.size as u32);
            jei::screen_grid(id, menu.as_deref(), mod_grid)
        })
    })
}

/// The server's recipes came in: the next frame builds the viewer from them.
pub(super) fn recipes_received() {
    STATE.with(|s| s.borrow_mut().rebuild = true);
}

fn dispatch(game: &mut Game<'_>, event: Event) {
    let grid = open_grid();
    let recipe_key = game.native_mod_key(jei::ID, "recipe", "key.keyboard.r");
    let uses_key = game.native_mod_key(jei::ID, "uses", "key.keyboard.u");
    let give = jei::give(game.native_setting(jei::ID, "give").as_deref());
    let lang = lang(game);
    let ctx = Context {
        recipe_key: &recipe_key,
        uses_key: &uses_key,
        give,
        // `native_give` succeeds in creative only: try, and fall back below.
        creative: give,
        grid,
    };

    let mut fallback = false;
    for out in handle(&event, &ctx, lang) {
        if let Out::Give { item, count } = out {
            if game.native_give(&item, count).is_err() {
                fallback = true;
            }
            continue;
        }
        if let Err(error) = apply(game, out) {
            game.native_chat(&format!("JEI: {error}"));
        }
    }
    if fallback {
        // Survival: the same click opens the recipes, as JEI without cheat mode.
        let ctx = Context {
            give: false,
            creative: false,
            ..ctx
        };
        for out in handle(&event, &ctx, lang) {
            if let Err(error) = apply(game, out) {
                game.native_chat(&format!("JEI: {error}"));
            }
        }
    }
}

fn handle(event: &Event, ctx: &Context<'_>, lang: jei::Lang) -> Vec<Out> {
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let Some(viewer) = state.viewer_mut() else {
            return Vec::new();
        };
        viewer.set_lang(lang);
        viewer.handle(event.clone(), ctx)
    })
}

/// The viewer's size, for `/jei`; none outside a world.
pub(super) fn counts() -> Option<(usize, usize)> {
    STATE.with(|s| {
        s.borrow()
            .viewer()
            .map(|viewer| (viewer.item_count(), viewer.recipe_count()))
    })
}

fn apply(game: &mut Game<'_>, out: Out) -> Result<(), String> {
    match out {
        Out::Items(stacks) => {
            game.native_panel_items(jei::ID, panel_stacks(stacks));
            Ok(())
        }
        Out::Recipes {
            title,
            tabs,
            tab,
            views,
            names,
            catalysts,
            subtitle,
        } => {
            if config().docked {
                game.native_panel_recipes(
                    jei::ID,
                    &title,
                    to_native(tabs),
                    tab,
                    views.into_iter().flat_map(native_views).collect(),
                );
                return Ok(());
            }
            if views.is_empty() {
                close_overlay(game);
                return Ok(());
            }
            let (open, same) = STATE.with(|s| {
                let state = s.borrow();
                let open = state.overlay.as_ref();
                (
                    open.map(|o| o.own_screen),
                    open.is_some_and(|o| o.title == title && o.tab == tab as usize),
                )
            });
            // Nothing open (a lookup from the world): JEI's own screen holds it.
            let own_screen = match open {
                Some(own) => own,
                None => {
                    let nothing_open = STATE.with(|s| s.borrow().screen.is_none());
                    nothing_open && game.native_mod_screen_open(jei::ID)
                }
            };
            // Notes wrap to the window's width (196 at most, less its margins), so the
            // geometry, which counts their lines, sizes them.
            let mut views = views;
            for view in &mut views {
                if !view.note.is_empty() {
                    view.note =
                        jei::wrap(&view.note, 180.0, |t| game.native_text_width(t)).join("\n");
                }
            }
            STATE.with(|s| {
                let mut state = s.borrow_mut();
                let page = match &state.overlay {
                    Some(o) if same => o.page,
                    _ => 0,
                };
                state.overlay = Some(Overlay {
                    title,
                    tabs,
                    names,
                    tab: tab as usize,
                    views,
                    catalysts,
                    subtitle,
                    page,
                    own_screen,
                });
            });
            Ok(())
        }
        Out::Give { item, count } => game.native_give(&item, count),
        Out::Transfer { book_id } => {
            game.native_transfer_recipe(book_id);
            Ok(())
        }
        Out::Say(text) => {
            game.native_chat(&text);
            Ok(())
        }
    }
}

/// The game's stacks. An empty list keeps one invisible slot: the panel (and
/// its search box and events) goes away when the list is empty.
fn panel_stacks(stacks: Vec<Stack>) -> Vec<NativeStack> {
    if stacks.is_empty() {
        return vec![NativeStack {
            item: "minecraft:air".to_owned(),
            count: 0,
            components: String::new(),
        }];
    }
    to_native(stacks)
}

fn to_native(stacks: Vec<Stack>) -> Vec<NativeStack> {
    stacks
        .into_iter()
        .map(|s| NativeStack {
            item: s.item,
            count: s.count,
            components: s.components,
        })
        .collect()
}

/// A bookmark as the game's stack, with the components `/give` reads.
fn item_stack(stack: &Stack) -> Option<ItemStack> {
    let (damage, extra) = if stack.components.is_empty() {
        (0, None)
    } else {
        let tag = horizon_client::item_data::command_components(&stack.components).ok()?;
        horizon_client::item_data::decode_components(Some(&tag))
    };
    Some(ItemStack {
        item: stack.item.as_str().into(),
        count: stack.count.clamp(1, 99),
        damage,
        extra,
    })
}

/// A view as the item panel draws it: a category recipe becomes one
/// "input -> output" per output, its chance and count in the note.
fn native_views(view: View) -> Vec<NativeRecipeView> {
    if view.layout != Layout::Chance {
        return vec![native_view(view)];
    }
    let input = view.slots.first().cloned().unwrap_or_default();
    view.chances
        .iter()
        .map(|c| {
            native_view(View {
                layout: Layout::Single,
                width: 1,
                height: 1,
                slots: vec![input.clone()],
                results: vec![c.stack.clone()],
                note: format!(
                    "{} {}",
                    jei::percent(c.chance),
                    jei::count_range(c.min, c.max)
                )
                .trim_end()
                .to_owned(),
                action: None,
                chances: Vec::new(),
            })
        })
        .collect()
}

fn native_view(view: View) -> NativeRecipeView {
    NativeRecipeView {
        layout: match view.layout {
            Layout::Grid => NativeRecipeLayout::Grid,
            Layout::Cooking => NativeRecipeLayout::Cooking,
            Layout::Single => NativeRecipeLayout::Single,
            Layout::Smithing => NativeRecipeLayout::Smithing,
            Layout::Brewing => NativeRecipeLayout::Brewing,
            Layout::Chance => NativeRecipeLayout::Single,
        },
        width: view.width,
        height: view.height,
        slots: view.slots.into_iter().map(to_native).collect(),
        results: to_native(view.results),
        note: view.note,
        action: view.action,
    }
}

fn convert_event(event: &NativePanelEvent) -> Option<Event> {
    Some(match event {
        NativePanelEvent::Search(text) => Event::Search(text.clone()),
        NativePanelEvent::Clicked { index, right } => Event::Clicked {
            index: *index as usize,
            right: *right,
        },
        NativePanelEvent::Key { key, stack } => Event::Key {
            key: key.clone(),
            stack: stack_of(stack),
        },
        NativePanelEvent::Tab(tab) => Event::Tab(*tab as usize),
        NativePanelEvent::Back => Event::Back,
        NativePanelEvent::SlotClicked { stack, right, .. } => Event::SlotClicked {
            stack: stack_of(stack),
            right: *right,
        },
        NativePanelEvent::Action(index) => Event::Action(*index as usize),
        _ => return None,
    })
}

fn stack_of(stack: &NativeStack) -> Stack {
    Stack {
        item: stack.item.clone(),
        count: stack.count,
        components: stack.components.clone(),
    }
}

fn items(catalog: &NativeRecipes) -> Vec<Item> {
    catalog
        .items
        .iter()
        .map(|item| Item {
            id: item.id.clone(),
            name: item.name.clone(),
            tags: item.tags.clone(),
        })
        .collect()
}

fn recipes(catalog: &NativeRecipes) -> Vec<Recipe> {
    catalog
        .recipes
        .iter()
        // A recipe with no ingredients says nothing in a list of recipes.
        .filter(|recipe| !recipe.slots.is_empty())
        .map(|recipe| Recipe {
            kind: kind(recipe.kind),
            width: recipe.width,
            height: recipe.height,
            slots: ingredient_slots(&recipe.slots),
            results: ingredient_list(&recipe.results),
            cooking_time: recipe.cooking_time,
            experience: recipe.experience,
            book_id: recipe.book_id,
        })
        .collect()
}

fn ingredient_slots(slots: &[Vec<NativeRecipeStack>]) -> Vec<Vec<Ingredient>> {
    slots.iter().map(|slot| ingredient_list(slot)).collect()
}

fn ingredient_list(stacks: &[NativeRecipeStack]) -> Vec<Ingredient> {
    stacks
        .iter()
        .map(|stack| Ingredient {
            item: stack.item,
            count: stack.count,
            components: stack.components.clone(),
        })
        .collect()
}

fn kind(kind: NativeRecipeKind) -> Kind {
    match kind {
        NativeRecipeKind::CraftingShaped => Kind::CraftingShaped,
        NativeRecipeKind::CraftingShapeless => Kind::CraftingShapeless,
        NativeRecipeKind::Smelting => Kind::Smelting,
        NativeRecipeKind::Blasting => Kind::Blasting,
        NativeRecipeKind::Smoking => Kind::Smoking,
        NativeRecipeKind::Campfire => Kind::Campfire,
        NativeRecipeKind::Stonecutting => Kind::Stonecutting,
        NativeRecipeKind::Smithing => Kind::Smithing,
        NativeRecipeKind::Brewing => Kind::Brewing,
    }
}

// ---------------------------------------------------------------- the recipe screen

const PANEL_BG: u32 = 0xFF_C6_C6_C6;
const PANEL_LIGHT: u32 = 0xFF_FF_FF_FF;
const PANEL_DARK: u32 = 0xFF_55_55_55;
const OUTLINE: u32 = 0xFF_00_00_00;
const SLOT_BG: u32 = 0xFF_8B_8B_8B;
const SLOT_DARK: u32 = 0xFF_37_37_37;
const TEXT: u32 = 0x40_40_40;
const DIM: u32 = 0xA0_10_10_10;

/// The recipe screen's parts now, from the GUI's size.
fn overlay_geom(overlay: &Overlay, gui: [f32; 2]) -> geo::Geom {
    geo::fitted(
        gui,
        overlay.tabs.len(),
        overlay.tab,
        overlay.catalysts.len(),
        &overlay.views,
    )
}

/// Closes the recipe screen (and JEI's own screen under it).
fn close_overlay(game: &mut Game<'_>) {
    let closed = STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.overlay_pressed = false;
        state.overlay.take()
    });
    if closed.is_some_and(|o| o.own_screen) {
        game.native_mod_screen_close(jei::ID);
    }
}

/// The stack a slot of alternatives shows now: one a second, as JEI cycles.
fn cycled(stacks: &[Stack]) -> Option<&Stack> {
    if stacks.is_empty() {
        return None;
    }
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs()) as usize;
    stacks.get(secs % stacks.len())
}

/// What is under the pointer on the recipe screen.
#[derive(Clone, Debug, PartialEq)]
enum Hit {
    Tab(usize),
    PrevTab,
    NextTab,
    PrevPage,
    NextPage,
    /// A stack: an input, a result, an output or a catalyst. `output` is
    /// the category output's index in its view, for its tooltip.
    Stack {
        stack: Stack,
        view: Option<usize>,
        output: Option<usize>,
    },
    /// The "+" of a view, and whether it is on.
    Plus(usize, bool),
    /// Somewhere else on the window.
    Inside,
}

/// Every hit place of the page shown, in drawing order.
fn overlay_places(overlay: &Overlay, g: &geo::Geom) -> Vec<(Hit, [f32; 4])> {
    let mut out = Vec::new();
    for (i, rect) in &g.tabs {
        out.push((Hit::Tab(*i), *rect));
    }
    if overlay.tabs.len() > 1 {
        out.push((Hit::PrevTab, g.prev_tab));
        out.push((Hit::NextTab, g.next_tab));
    }
    let pages = geo::pages(&overlay.views, g.content);
    if pages.len() > 1 {
        out.push((Hit::PrevPage, g.prev_page));
        out.push((Hit::NextPage, g.next_page));
    }
    for (stack, rect) in overlay.catalysts.iter().zip(&g.catalysts) {
        out.push((
            Hit::Stack {
                stack: stack.clone(),
                view: None,
                output: None,
            },
            *rect,
        ));
    }
    let page = overlay.page.min(pages.len().saturating_sub(1));
    let Some(pieces) = pages.get(page) else {
        return out;
    };
    for (n, at) in geo::layout(&overlay.views, pieces, g.content) {
        let piece = &pieces[n];
        let view = &overlay.views[piece.view];
        let placed = geo::place(view, piece, at, g.content[2]);
        for (part, rect) in placed.slots {
            let (stack, output) = match part {
                geo::Part::Input(i) => (view.slots.get(i).and_then(|s| cycled(s)), None),
                geo::Part::Result(i) => (view.results.get(i), Some(i)),
            };
            if let Some(stack) = stack {
                out.push((
                    Hit::Stack {
                        stack: stack.clone(),
                        view: Some(piece.view),
                        output: output.filter(|_| view.layout == Layout::Chance),
                    },
                    rect,
                ));
            }
        }
        if let (Some(rect), Some(on)) = (placed.button, view.action) {
            out.push((Hit::Plus(piece.view, on), rect));
        }
    }
    out
}

/// The hit under `point`, if the point is on the recipe screen at all.
fn overlay_hit(overlay: &Overlay, gui: [f32; 2], point: [f32; 2]) -> Option<Hit> {
    let g = overlay_geom(overlay, gui);
    let places = overlay_places(overlay, &g);
    if let Some((hit, _)) = places.iter().rev().find(|(_, r)| jei::inside(point, *r)) {
        return Some(hit.clone());
    }
    let on = jei::inside(point, g.panel)
        || (!g.catalysts.is_empty() && jei::inside(point, g.catalyst_panel));
    on.then_some(Hit::Inside)
}

fn frame_box(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch, r: [f32; 4]) {
    let [x, y, w, h] = r;
    game.native_fill(batch, [x - 1.0, y - 1.0, w + 2.0, h + 2.0], OUTLINE);
    game.native_fill(batch, r, PANEL_BG);
    game.native_fill(batch, [x, y, w - 1.0, 1.0], PANEL_LIGHT);
    game.native_fill(batch, [x, y, 1.0, h - 1.0], PANEL_LIGHT);
    game.native_fill(batch, [x + 1.0, y + h - 1.0, w - 1.0, 1.0], PANEL_DARK);
    game.native_fill(batch, [x + w - 1.0, y + 1.0, 1.0, h - 1.0], PANEL_DARK);
}

fn slot_box(game: &Game<'_>, batch: &mut horizon_client::hud::HudBatch, r: [f32; 4]) {
    let [x, y, w, h] = r;
    game.native_fill(batch, r, SLOT_BG);
    game.native_fill(batch, [x, y, w - 1.0, 1.0], SLOT_DARK);
    game.native_fill(batch, [x, y, 1.0, h - 1.0], SLOT_DARK);
    game.native_fill(batch, [x + 1.0, y + h - 1.0, w - 1.0, 1.0], PANEL_LIGHT);
    game.native_fill(batch, [x + w - 1.0, y + 1.0, 1.0, h - 1.0], PANEL_LIGHT);
}

fn button_box(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    r: [f32; 4],
    label: &str,
    on: bool,
    hovered: bool,
) {
    let fill = match (on, hovered) {
        (false, _) => 0xFF_6E_6E_6E,
        (true, true) => 0xFF_8C_9A_D8,
        (true, false) => 0xFF_A0_A0_A0,
    };
    game.native_fill(batch, r, OUTLINE);
    game.native_fill(
        batch,
        [r[0] + 1.0, r[1] + 1.0, r[2] - 2.0, r[3] - 2.0],
        fill,
    );
    let w = game.native_text_width(label);
    let color = if on { 0xFF_FF_FF } else { 0xA0_A0_A0 };
    game.native_text(
        batch,
        label,
        [(r[0] + (r[2] - w) / 2.0).round(), r[1] + (r[3] - 8.0) / 2.0],
        color,
        true,
    );
}

fn draw_stack(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    stack: &Stack,
    r: [f32; 4],
) {
    if let Some(item) = item_stack(stack) {
        let count = (stack.count > 1).then(|| stack.count.to_string());
        game.native_gui_draw_item_with(batch, &item, [r[0] + 1.0, r[1] + 1.0], count.as_deref());
    }
}

/// The text centred on `x`, cut with "..." to fit `max`.
fn centred(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    text: &str,
    x: f32,
    y: f32,
    max: f32,
    rgb: u32,
) {
    let mut text = text.to_owned();
    while game.native_text_width(&text) > max && text.chars().count() > 3 {
        let keep: String = text.chars().take(text.chars().count() - 4).collect();
        text = format!("{keep}...");
    }
    let w = game.native_text_width(&text);
    game.native_text(batch, &text, [(x - w / 2.0).round(), y], rgb, false);
}

/// The recipe screen, over the screen at `rect` (`mouse`: the pointer).
fn draw_overlay(
    game: &Game<'_>,
    batch: &mut horizon_client::hud::HudBatch,
    rect: [f32; 4],
    mouse: [f32; 2],
    overlay: &Overlay,
) {
    let gui = [batch.width, batch.height];
    let g = overlay_geom(overlay, gui);
    let lang = lang(game);
    // The screen under it dims, as under a vanilla screen opened over another.
    game.native_fill(batch, rect, DIM);
    let hit = overlay_hit(overlay, gui, mouse);
    // Tabs: the selected one joins the window.
    for (i, rect) in &g.tabs {
        let selected = *i == overlay.tab;
        let r = if selected {
            [rect[0], rect[1], rect[2] - 1.0, rect[3] + 2.0]
        } else {
            [rect[0], rect[1] + 2.0, rect[2] - 1.0, rect[3] - 2.0]
        };
        frame_box(game, batch, r);
        if !selected {
            game.native_fill(batch, [r[0], r[1], r[2], r[3]], 0x30_00_00_00);
        }
        if let Some(icon) = overlay.tabs.get(*i) {
            draw_stack(game, batch, icon, [r[0] + 3.0, r[1] + 2.0, 18.0, 18.0]);
        }
    }
    frame_box(game, batch, g.panel);
    if !g.catalysts.is_empty() {
        frame_box(game, batch, g.catalyst_panel);
        for (stack, rect) in overlay.catalysts.iter().zip(&g.catalysts) {
            slot_box(game, batch, *rect);
            draw_stack(game, batch, stack, *rect);
        }
    }
    let name = overlay.names.get(overlay.tab).cloned().unwrap_or_default();
    centred(
        game,
        batch,
        &name,
        g.title[0],
        g.title[1],
        g.panel[2] - 44.0,
        TEXT,
    );
    if overlay.tabs.len() > 1 {
        button_box(
            game,
            batch,
            g.prev_tab,
            "<",
            true,
            hit == Some(Hit::PrevTab),
        );
        button_box(
            game,
            batch,
            g.next_tab,
            ">",
            true,
            hit == Some(Hit::NextTab),
        );
    }
    let pages = geo::pages(&overlay.views, g.content);
    let page = overlay.page.min(pages.len().saturating_sub(1));
    // A one-page category shows its whole subtitle there; more pages, the page too.
    let line = match (overlay.subtitle.is_empty(), pages.len() > 1) {
        (true, _) => lang.page_of(page, pages.len()),
        (false, false) => overlay.subtitle.clone(),
        (false, true) => format!("{}  {}", overlay.subtitle, lang.page_of(page, pages.len())),
    };
    // The page arrows flank the line only when there are pages to turn.
    let room = if pages.len() > 1 {
        g.panel[2] - 44.0
    } else {
        g.panel[2] - 12.0
    };
    centred(
        game,
        batch,
        &line,
        g.page_text[0],
        g.page_text[1],
        room,
        0x60_60_60,
    );
    if pages.len() > 1 {
        button_box(
            game,
            batch,
            g.prev_page,
            "<",
            true,
            hit == Some(Hit::PrevPage),
        );
        button_box(
            game,
            batch,
            g.next_page,
            ">",
            true,
            hit == Some(Hit::NextPage),
        );
    }
    if let Some(pieces) = pages.get(page) {
        for (n, at) in geo::layout(&overlay.views, pieces, g.content) {
            let piece = &pieces[n];
            let view = &overlay.views[piece.view];
            let placed = geo::place(view, piece, at, g.content[2]);
            for (part, rect) in &placed.slots {
                slot_box(game, batch, *rect);
                let stack = match part {
                    geo::Part::Input(i) => view.slots.get(*i).and_then(|s| cycled(s)),
                    geo::Part::Result(i) => view.results.get(*i),
                };
                if let Some(stack) = stack {
                    draw_stack(game, batch, stack, *rect);
                }
                if let (geo::Part::Result(i), Layout::Chance) = (part, view.layout)
                    && let Some(c) = view.chances.get(*i)
                {
                    let mark = if c.notes.is_empty() { "" } else { "*" };
                    let x = rect[0] + geo::SLOT + 2.0;
                    game.native_text(
                        batch,
                        &format!("{}{mark}", jei::percent(c.chance)),
                        [x, rect[1] + 1.0],
                        TEXT,
                        false,
                    );
                    let range = jei::count_range(c.min, c.max);
                    if !range.is_empty() {
                        game.native_text(batch, &range, [x, rect[1] + 10.0], 0x70_70_70, false);
                    }
                }
            }
            if let Some([x, y, w, h]) = placed.arrow {
                let mid = y + h / 2.0;
                game.native_fill(batch, [x, mid - 2.0, w - 6.0, 4.0], 0xFF_8B_8B_8B);
                game.native_quad(
                    batch,
                    [
                        [x + w - 7.0, y],
                        [x + w, mid],
                        [x + w, mid],
                        [x + w - 7.0, y + h],
                    ],
                    0xFF_8B_8B_8B,
                );
            }
            if let (Some(rect), Some(on)) = (placed.button, view.action) {
                let hovered = hit == Some(Hit::Plus(piece.view, on));
                button_box(game, batch, rect, "+", on, hovered);
            }
            if !view.note.is_empty() && piece.outputs.start == 0 {
                for (k, text) in view.note.split('\n').enumerate() {
                    game.native_text(
                        batch,
                        text,
                        [placed.notes[0], placed.notes[1] + k as f32 * geo::LINE],
                        0x50_50_50,
                        false,
                    );
                }
            }
        }
    }
    // The tooltip of what the pointer is on.
    let lines: Vec<String> = match &hit {
        Some(Hit::Stack {
            stack,
            view,
            output,
        }) => overlay_stack_lines(game, overlay, stack, *view, *output),
        Some(Hit::Plus(_, on)) => vec![format!("§f{}", lang.transfer_hint(*on))],
        Some(Hit::Tab(i)) => overlay
            .names
            .get(*i)
            .map(|n| vec![format!("§f{n}")])
            .unwrap_or_default(),
        _ => Vec::new(),
    };
    if !lines.is_empty() {
        let texts: Vec<horizon_client::text::Text> = lines
            .iter()
            .map(|l| horizon_client::text::Text::literal(l.as_str()).resolve_legacy())
            .collect();
        game.native_gui_tooltip(batch, &texts, mouse);
    }
}

/// A stack's tooltip on the recipe screen: its name, a category output's
/// chance, count and notes, and JEI's hints.
fn overlay_stack_lines(
    game: &Game<'_>,
    overlay: &Overlay,
    stack: &Stack,
    view: Option<usize>,
    output: Option<usize>,
) -> Vec<String> {
    let lang = lang(game);
    let (name, counts) = STATE.with(|s| {
        let state = s.borrow();
        state
            .viewer()
            .map_or((stack.item.clone(), Counts::default()), |v| {
                (
                    v.item_name(&stack.item),
                    v.counts(&stack.item, &stack.components),
                )
            })
    });
    let mut lines = vec![format!("§f{name}")];
    if let (Some(v), Some(o)) = (view, output)
        && let Some(c) = overlay.views.get(v).and_then(|v| v.chances.get(o))
    {
        let range = jei::count_range(c.min, c.max);
        lines.push(format!(
            "§e{}{}",
            jei::percent(c.chance),
            if range.is_empty() {
                String::new()
            } else {
                format!(" {range}")
            }
        ));
        for note in &c.notes {
            lines.push(format!("§7{note}"));
        }
    }
    let marked = STATE.with(|s| s.borrow().bookmarks.position(stack).is_some());
    for (text, _) in jei::tooltip_lines(counts, marked, lang) {
        lines.push(format!("§8{text}"));
    }
    lines
}

/// A press on the recipe screen: it takes every press on its window, and
/// the presses on the screen under it (it is modal), but not the item list
/// beside the screen or the bookmarks.
fn overlay_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    button: i32,
    modifiers: u32,
    canceled: &mut bool,
) {
    let _ = modifiers;
    let gui = STATE.with(|s| s.borrow().gui);
    let hit = STATE.with(|s| {
        let state = s.borrow();
        state
            .overlay
            .as_ref()
            .map(|o| overlay_hit(o, gui, screen.mouse))
    });
    let Some(hit) = hit else {
        return;
    };
    let Some(hit) = hit else {
        // Outside the window: the item list (right of the screen's window)
        // still takes clicks; the screen under the window doesn't.
        let beside = screen.mouse[0] >= screen.rect[0] + screen.rect[2];
        if !(beside && jei::overlay_screen(screen.id)) {
            *canceled = true;
            STATE.with(|s| s.borrow_mut().overlay_pressed = true);
        }
        return;
    };
    *canceled = true;
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        state.overlay_pressed = true;
        let tabs = state.overlay.as_ref().map_or(0, |o| o.tabs.len());
        let tab = state.overlay.as_ref().map_or(0, |o| o.tab);
        let pages = state.overlay.as_ref().map_or(1, |o| {
            geo::pages(&o.views, overlay_geom(o, gui).content)
                .len()
                .max(1)
        });
        match hit {
            Hit::Tab(i) => state.pending.push(Event::Tab(i)),
            Hit::PrevTab if tabs > 0 => state.pending.push(Event::Tab((tab + tabs - 1) % tabs)),
            Hit::NextTab if tabs > 0 => state.pending.push(Event::Tab((tab + 1) % tabs)),
            Hit::PrevPage | Hit::NextPage => {
                if let Some(o) = state.overlay.as_mut() {
                    let page = o.page.min(pages - 1);
                    o.page = if hit == Hit::NextPage {
                        (page + 1) % pages
                    } else {
                        (page + pages - 1) % pages
                    };
                }
            }
            Hit::Stack { stack, .. } if button == 0 || button == 1 => {
                state.pending.push(Event::Open {
                    stack,
                    uses: button == 1,
                });
            }
            Hit::Plus(view, true) if button == 0 => {
                // JEI moves the items and goes back to the screen they went to.
                state.pending.push(Event::Action(view));
                state.pending.push(Event::Close);
            }
            _ => {}
        }
    });
    let _ = game;
}

/// The wheel over the recipe screen turns its pages.
pub(super) fn screen_mouse_scrolled(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    scroll: [f64; 2],
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    STATE.with(|s| {
        let mut state = s.borrow_mut();
        let gui = state.gui;
        let Some(o) = state.overlay.as_mut() else {
            return;
        };
        if overlay_hit(o, gui, screen.mouse).is_none() || scroll[1] == 0.0 {
            return;
        }
        *canceled = true;
        let pages = geo::pages(&o.views, overlay_geom(o, gui).content)
            .len()
            .max(1);
        let page = o.page.min(pages - 1);
        o.page = if scroll[1] < 0.0 {
            (page + 1).min(pages - 1)
        } else {
            page.saturating_sub(1)
        };
    });
}

/// Keys on the recipe screen: Escape and Backspace go back a page (closing
/// it at the first), and the viewer's keys (recipes, uses, bookmark) act on
/// the stack under the pointer.
pub(super) fn screen_key_pressed(
    game: &Game<'_>,
    screen: &NativeScreenView<'_>,
    key: &str,
    _repeat: bool,
    _modifiers: u32,
    canceled: &mut bool,
) {
    if !game.native_on(jei::ID) {
        return;
    }
    let gui = STATE.with(|s| s.borrow().gui);
    let hit = STATE.with(|s| {
        let state = s.borrow();
        state
            .overlay
            .as_ref()
            .map(|o| overlay_hit(o, gui, screen.mouse))
    });
    let Some(hit) = hit else {
        return;
    };
    if key == "key.keyboard.escape" || (key == "key.keyboard.backspace" && hit.is_some()) {
        *canceled = true;
        pending(Event::Back);
        return;
    }
    let recipe = game.native_mod_key(jei::ID, "recipe", "key.keyboard.r");
    let uses = game.native_mod_key(jei::ID, "uses", "key.keyboard.u");
    let mark = game.native_mod_key(jei::ID, "bookmark", jei::BOOKMARK_KEY);
    if ![&recipe, &uses, &mark].contains(&&key.to_owned()) || hit.is_none() {
        return;
    }
    // Over the window, its keys never reach the slots hidden under it.
    *canceled = true;
    let Some(Hit::Stack { stack, .. }) = hit else {
        return;
    };
    if key == mark {
        STATE.with(|s| s.borrow_mut().to_bookmark.push(stack));
    } else {
        pending(Event::Open {
            stack,
            uses: key == uses,
        });
    }
}

/// The screen under the recipe screen closed (or JEI's own did): so does
/// the recipe screen, and the viewer forgets its history.
pub(super) fn screen_closing(game: &Game<'_>, _screen: &NativeScreenView<'_>) {
    if !game.native_on(jei::ID) {
        return;
    }
    let open = STATE.with(|s| {
        let mut state = s.borrow_mut();
        if let Some(o) = state.overlay.as_mut() {
            // JEI's own screen is closing: nothing to close after it.
            o.own_screen = false;
            true
        } else {
            false
        }
    });
    if open {
        pending(Event::Close);
    }
}

/// The settings page's "Recipes" button: the recipe screen or the panel.
pub(super) fn toggle_docked(game: &mut Game<'_>) -> Config {
    let mut config = config();
    config.docked = !config.docked;
    store_config(config);
    close_overlay(game);
    dispatch(game, Event::Close);
    config
}
