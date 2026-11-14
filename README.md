# JEI for Horizon

An item and recipe viewer for [Horizon](https://playhorizon.dev), the Rust
reimplementation of Minecraft: Java Edition 26.3. It is a port of
[JustEnoughItems](https://github.com/mezz/JustEnoughItems) (JEI), rebuilt on
Horizon's native hook points.

Status: **0.2.1 "Overlay and bookmarks"**. On top of 0.1's panel viewer (every
item, JEI's search grammar, R/U over any stack, a tab per station, recipe
pages, "+", creative give, English and Spanish), the mod now draws over
container screens:

- **Item tooltips** say what makes an item and what uses it (`R: Recipes (12)`,
  `U: Uses (3)`) and mark the bookmarked ones, in the game's own tooltips,
  the panel's included.
- **Subtypes**: the potions, tipped arrows and other stacks the recipe tables
  name with components are items of their own in the list.
- **Bookmarks**: the **A** key bookmarks the stack under the pointer; a bar of
  two columns of twelve sits at the screen's top left corner, left click opens
  a stack's recipes, right click its uses, shift or the middle button takes the
  bookmark away, and dragging reorders it. The bar is kept in
  `config/jei/bookmarks.txt` and comes back with the next world.
- **Config**: `config/jei/jei.cfg` holds the list's order (`/jei sort name|id`)
  and the toggles on the mod's settings page (tooltip hints, the bar, where
  recipes show).
- **Recipe screen** (unreleased): R, U and clicks open JEI's own window over the
  open screen, with tabs, catalysts, pages and a "+" that fills the grid under
  it with no recipe-book unlock. Escape or Backspace goes back. The 0.2 panel
  view is a setting.

See [`docs/jei-parity.md`](docs/jei-parity.md) for the feature-by-feature state
against JEI.

Not there yet: click on a container slot (only R/U), the creative inventory
panel, the non-recipe categories (anvil, grindstone, compost, fuel), shift on
"+", recipe transfer into furnaces, and the `$tooltip` / `%creative tab`
searches. Some of those need new Horizon hooks; [`docs/roadmap.md`](docs/roadmap.md)
lists them.

## Install

From the registry: Mods → JEI → Install, then Settings → Game → Native mods
(experimental) → Build, and play the **Native mods** instance.

From a clone of this folder:

```sh
horizon-launcher native-build /path/to/horizon-jei/mixins/jei --trust-native-code
```

The first build downloads Rust and takes 20-30 minutes; later builds recompile
only the game. Native mods are not sandboxed: build only code you trust.

## Releases

A release is a `.hnmod` built from a commit of this repository with
`native/tools/native.py package`, checked with `cargo horizon-mod check`, and
signed with minisign. The public key is [`horizon-jei.pub`](horizon-jei.pub)
(key id `FC3386F1F51ECF6A`); the private key is not in the repository, and
every later version is signed with it, so nobody else can publish an update
under this project's name. The signature of each release goes into the
registry entry, not into this repository.

## Develop

```sh
cargo test                       # the mod's own logic, no game

# From a clone of https://github.com/horizon-rust/horizon-mods:
python3 native/tools/native.py check /path/to/horizon-jei
python3 native/tools/native.py check /path/to/horizon-jei --horizon latest
python3 native/tools/native.py package /path/to/horizon-jei
cargo horizon-mod check native/target/jei-<version>.hnmod
```

In game: Options → Mods → JEI, and `/jei`, `/jei sort`, `/jei bookmarks clear`,
`/jei config`.

## Layout

```
Cargo.toml                 the mod's tests (lib = mixins/jei/runtime.rs)
compatibility.json         game 26.3, hook points 313, the tested revision
mixins/jei/
  horizon-mixin.toml       the bundle: modules and HEAD hooks
  runtime.rs               the core (becomes crate::game::jei)
  glue.rs                  the game's tables and the panel in and out
  screen_post.rs tooltip.rs mouse_press.rs mouse_release.rs mouse_drag.rs
                           the overlay: the bar, its tooltip and its clicks
  list.rs settings.rs click.rs command.rs event.rs frame.rs
                           one helper per hook
```

## License

MIT. See `LICENSE` and `NOTICE` (attribution to JEI, © mezz).
Not affiliated with mezz, Mojang, Microsoft or the Horizon project.
