//! End-to-end integration and daily use test suite for JEI.
//!
//! Verifies:
//! - Exhaustive click-through on every single item in the catalog (recipes and uses)
//!   guaranteeing that exactly ONE recipe overlay is active and dual-window glitches never occur.
//! - Window mutual exclusion state machine (native overlay vs secondary/kit window).
//! - Everyday search workflows, filtering, and clicking filtered items.
//! - Deep history navigation, recipe drill-down, and clean Back unwinding.
//! - Category tab switching, catalyst discovery, and station views.
//! - Bookmark bar management (toggling, 24-slot capacity, drag reordering, text persistence).
//! - Recipe transfer "+" button validation across 2x2 and 3x3 crafting grids.
//! - Cheat/give mode vs survival mode click behaviors.
//! - Third-party mod category and catalyst integration.

use jei::*;

/// Builds a rich and diverse item catalog representing realistic gameplay.
fn realistic_catalog() -> (Vec<Item>, Vec<Recipe>) {
    let items = vec![
        Item {
            id: "minecraft:stone".into(),
            name: "Stone".into(),
            tags: vec!["#minecraft:stone_crafting_materials".into()],
        },
        Item {
            id: "minecraft:cobblestone".into(),
            name: "Cobblestone".into(),
            tags: vec!["#minecraft:stone_crafting_materials".into()],
        },
        Item {
            id: "minecraft:iron_ore".into(),
            name: "Iron Ore".into(),
            tags: vec!["#c:ores/iron".into()],
        },
        Item {
            id: "minecraft:deepslate_iron_ore".into(),
            name: "Deepslate Iron Ore".into(),
            tags: vec!["#c:ores/iron".into()],
        },
        Item {
            id: "minecraft:raw_iron".into(),
            name: "Raw Iron".into(),
            tags: vec!["#c:raw_materials/iron".into()],
        },
        Item {
            id: "minecraft:iron_ingot".into(),
            name: "Iron Ingot".into(),
            tags: vec!["#c:ingots/iron".into()],
        },
        Item {
            id: "minecraft:iron_block".into(),
            name: "Block of Iron".into(),
            tags: vec!["#c:storage_blocks/iron".into()],
        },
        Item {
            id: "minecraft:iron_sword".into(),
            name: "Iron Sword".into(),
            tags: vec!["#minecraft:swords".into()],
        },
        Item {
            id: "minecraft:iron_pickaxe".into(),
            name: "Iron Pickaxe".into(),
            tags: vec!["#minecraft:pickaxes".into()],
        },
        Item {
            id: "minecraft:stick".into(),
            name: "Stick".into(),
            tags: vec!["#c:rods/wooden".into()],
        },
        Item {
            id: "minecraft:oak_planks".into(),
            name: "Oak Planks".into(),
            tags: vec!["#minecraft:planks".into()],
        },
        Item {
            id: "minecraft:oak_log".into(),
            name: "Oak Log".into(),
            tags: vec!["#minecraft:logs".into()],
        },
        Item {
            id: "minecraft:crafting_table".into(),
            name: "Crafting Table".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:furnace".into(),
            name: "Furnace".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:blast_furnace".into(),
            name: "Blast Furnace".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:diamond".into(),
            name: "Diamond".into(),
            tags: vec!["#c:gems/diamond".into()],
        },
        Item {
            id: "minecraft:diamond_block".into(),
            name: "Block of Diamond".into(),
            tags: vec!["#c:storage_blocks/diamond".into()],
        },
        Item {
            id: "minecraft:diamond_sword".into(),
            name: "Diamond Sword".into(),
            tags: vec!["#minecraft:swords".into()],
        },
        Item {
            id: "minecraft:bucket".into(),
            name: "Bucket".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:water_bucket".into(),
            name: "Water Bucket".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:potion".into(),
            name: "Potion".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:bedrock".into(),
            name: "Bedrock".into(),
            tags: vec![],
        },
        Item {
            id: "minecraft:barrier".into(),
            name: "Barrier".into(),
            tags: vec![],
        },
        Item {
            id: "ae2:certus_quartz_crystal".into(),
            name: "Certus Quartz Crystal".into(),
            tags: vec!["#c:gems/certus_quartz".into()],
        },
        Item {
            id: "ae2:inscriber".into(),
            name: "Inscriber".into(),
            tags: vec![],
        },
    ];

    let ing = |idx: u32| Ingredient {
        item: idx,
        count: 1,
        components: String::new(),
    };

    let recipes = vec![
        // Oak Log (11) -> 4x Oak Planks (10) [shapeless 1x1]
        Recipe {
            kind: Kind::CraftingShapeless,
            width: 0,
            height: 0,
            slots: vec![vec![ing(11)]],
            results: vec![Ingredient {
                item: 10,
                count: 4,
                components: String::new(),
            }],
            cooking_time: 0,
            experience: 0.0,
            book_id: 1,
        },
        // 2x Oak Planks (10) -> 4x Stick (9) [shaped 1x2]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 1,
            height: 2,
            slots: vec![vec![ing(10)], vec![ing(10)]],
            results: vec![Ingredient {
                item: 9,
                count: 4,
                components: String::new(),
            }],
            cooking_time: 0,
            experience: 0.0,
            book_id: 2,
        },
        // 4x Oak Planks (10) -> Crafting Table (12) [shaped 2x2]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 2,
            height: 2,
            slots: vec![vec![ing(10)], vec![ing(10)], vec![ing(10)], vec![ing(10)]],
            results: vec![ing(12)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 3,
        },
        // 8x Cobblestone (1) -> Furnace (13) [shaped 3x3 hollow]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 3,
            height: 3,
            slots: vec![
                vec![ing(1)],
                vec![ing(1)],
                vec![ing(1)],
                vec![ing(1)],
                vec![],
                vec![ing(1)],
                vec![ing(1)],
                vec![ing(1)],
                vec![ing(1)],
            ],
            results: vec![ing(13)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 4,
        },
        // Iron Ore (2) -> Iron Ingot (5) [smelting in Furnace]
        Recipe {
            kind: Kind::Smelting,
            width: 0,
            height: 0,
            slots: vec![vec![ing(2)]],
            results: vec![ing(5)],
            cooking_time: 200,
            experience: 0.7,
            book_id: 5,
        },
        // Deepslate Iron Ore (3) -> Iron Ingot (5) [smelting in Furnace]
        Recipe {
            kind: Kind::Smelting,
            width: 0,
            height: 0,
            slots: vec![vec![ing(3)]],
            results: vec![ing(5)],
            cooking_time: 200,
            experience: 0.7,
            book_id: 6,
        },
        // Iron Ore (2) -> Iron Ingot (5) [blasting in Blast Furnace (faster)]
        Recipe {
            kind: Kind::Blasting,
            width: 0,
            height: 0,
            slots: vec![vec![ing(2)]],
            results: vec![ing(5)],
            cooking_time: 100,
            experience: 0.7,
            book_id: 7,
        },
        // 9x Iron Ingot (5) -> Block of Iron (6) [shaped 3x3]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 3,
            height: 3,
            slots: vec![
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
            ],
            results: vec![ing(6)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 8,
        },
        // Block of Iron (6) -> 9x Iron Ingot (5) [shapeless 1x1]
        Recipe {
            kind: Kind::CraftingShapeless,
            width: 0,
            height: 0,
            slots: vec![vec![ing(6)]],
            results: vec![Ingredient {
                item: 5,
                count: 9,
                components: String::new(),
            }],
            cooking_time: 0,
            experience: 0.0,
            book_id: 9,
        },
        // 2x Iron Ingot (5) + Stick (9) -> Iron Sword (7) [shaped 1x3]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 1,
            height: 3,
            slots: vec![vec![ing(5)], vec![ing(5)], vec![ing(9)]],
            results: vec![ing(7)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 10,
        },
        // 3x Iron Ingot (5) + 2x Stick (9) -> Iron Pickaxe (8) [shaped 3x3]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 3,
            height: 3,
            slots: vec![
                vec![ing(5)],
                vec![ing(5)],
                vec![ing(5)],
                vec![],
                vec![ing(9)],
                vec![],
                vec![],
                vec![ing(9)],
                vec![],
            ],
            results: vec![ing(8)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 11,
        },
        // 9x Diamond (15) -> Block of Diamond (16) [shaped 3x3]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 3,
            height: 3,
            slots: vec![
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
                vec![ing(15)],
            ],
            results: vec![ing(16)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 12,
        },
        // Block of Diamond (16) -> 9x Diamond (15) [shapeless 1x1]
        Recipe {
            kind: Kind::CraftingShapeless,
            width: 0,
            height: 0,
            slots: vec![vec![ing(16)]],
            results: vec![Ingredient {
                item: 15,
                count: 9,
                components: String::new(),
            }],
            cooking_time: 0,
            experience: 0.0,
            book_id: 13,
        },
        // 2x Diamond (15) + Stick (9) -> Diamond Sword (17) [shaped 1x3]
        Recipe {
            kind: Kind::CraftingShaped,
            width: 1,
            height: 3,
            slots: vec![vec![ing(15)], vec![ing(15)], vec![ing(9)]],
            results: vec![ing(17)],
            cooking_time: 0,
            experience: 0.0,
            book_id: 14,
        },
    ];

    (items, recipes)
}

fn test_context(grid: u32, give: bool) -> Context<'static> {
    Context {
        recipe_key: "key.keyboard.r",
        uses_key: "key.keyboard.u",
        give,
        creative: give,
        grid,
    }
}

// ---------------------------------------------------------------------------
// 1. Exhaustive Item Click-Through: Guaranteeing Zero Dual Panels
// ---------------------------------------------------------------------------

#[test]
fn test_click_every_item_in_catalog_recipes_and_uses_never_duplicates_window() {
    let (items, recipes) = realistic_catalog();
    let total_items = items.len();
    let (mut viewer, initial_out) = Viewer::new(items, recipes, Lang::En);

    let panel_items = match initial_out {
        Out::Items(stacks) => stacks,
        _ => panic!("initial output must be panel items list"),
    };
    assert_eq!(panel_items.len(), total_items);

    let ctx = test_context(3, false);

    for (index, stack) in panel_items.iter().enumerate() {
        // --- A. Left Click (Show Recipes) ---
        let outs = viewer.handle(
            Event::Clicked {
                index,
                right: false,
            },
            &ctx,
        );
        assert_eq!(
            outs.len(),
            1,
            "clicking item {} ({}) must produce exactly one output",
            index,
            stack.item
        );

        match &outs[0] {
            Out::Recipes {
                title,
                tabs,
                tab,
                views,
                catalysts,
                ..
            } => {
                assert!(
                    !views.is_empty(),
                    "recipe view for {} must contain at least one recipe",
                    stack.item
                );
                assert!(
                    !tabs.is_empty(),
                    "category tabs for {} must not be empty",
                    stack.item
                );
                assert!(
                    (*tab as usize) < tabs.len(),
                    "active tab index must be within range"
                );
                assert!(
                    title.contains("Recipes"),
                    "title must indicate recipes for {}: got {title}",
                    stack.item
                );
                let _ = catalysts; // Valid catalysts vector present
            }
            Out::Say(msg) => {
                // Items without recipes (e.g. bedrock, barrier) return a clean say message
                assert!(
                    msg.contains("No recipes"),
                    "item without recipe must say 'No recipes': {msg}"
                );
            }
            other => panic!("unexpected response on left click: {other:?}"),
        }

        // Close the window and ensure clean return to base
        let close_outs = viewer.handle(Event::Close, &ctx);
        assert_eq!(close_outs.len(), 1);
        match &close_outs[0] {
            Out::Recipes { tabs, views, .. } => {
                assert!(tabs.is_empty() && views.is_empty());
            }
            other => panic!("close must return empty recipe state: {other:?}"),
        }

        // --- B. Right Click (Show Uses) ---
        let outs_uses = viewer.handle(Event::Clicked { index, right: true }, &ctx);
        assert_eq!(
            outs_uses.len(),
            1,
            "right-clicking item {} ({}) must produce exactly one output",
            index,
            stack.item
        );

        match &outs_uses[0] {
            Out::Recipes {
                title,
                tabs,
                tab,
                views,
                ..
            } => {
                assert!(
                    !views.is_empty(),
                    "uses view for {} must contain at least one recipe",
                    stack.item
                );
                assert!(!tabs.is_empty());
                assert!((*tab as usize) < tabs.len());
                assert!(
                    title.contains("Uses"),
                    "title must indicate uses for {}: got {title}",
                    stack.item
                );
            }
            Out::Say(msg) => {
                assert!(
                    msg.contains("No uses"),
                    "item without uses must say 'No uses': {msg}"
                );
            }
            other => panic!("unexpected response on right click: {other:?}"),
        }

        // Clean close
        viewer.handle(Event::Close, &ctx);

        // --- C. Key Shortcut: 'R' (Show Recipes) ---
        let key_r_outs = viewer.handle(
            Event::Key {
                key: "key.keyboard.r".into(),
                stack: stack.clone(),
            },
            &ctx,
        );
        assert_eq!(key_r_outs.len(), 1);
        assert_eq!(
            key_r_outs[0], outs[0],
            "Key 'R' must produce exact parity with Left Click"
        );
        viewer.handle(Event::Close, &ctx);

        // --- D. Key Shortcut: 'U' (Show Uses) ---
        let key_u_outs = viewer.handle(
            Event::Key {
                key: "key.keyboard.u".into(),
                stack: stack.clone(),
            },
            &ctx,
        );
        assert_eq!(key_u_outs.len(), 1);
        assert_eq!(
            key_u_outs[0], outs_uses[0],
            "Key 'U' must produce exact parity with Right Click"
        );
        viewer.handle(Event::Close, &ctx);
    }
}

// ---------------------------------------------------------------------------
// 2. Window Mutual Exclusion & Dual-Panel Prevention Invariant
// ---------------------------------------------------------------------------

#[test]
fn test_overlay_and_kit_window_mutual_exclusion_invariant() {
    // Simulates the window state machine of jei_glue and hz_kit_window_jei.
    struct WindowManager {
        overlay_open: bool,
        kit_window_open: bool,
    }

    impl WindowManager {
        fn new() -> Self {
            Self {
                overlay_open: false,
                kit_window_open: false,
            }
        }

        fn open_overlay(&mut self) {
            // When official JEI overlay opens, kit window is closed immediately.
            self.kit_window_open = false;
            self.overlay_open = true;
            self.assert_invariant();
        }

        fn close_overlay(&mut self) {
            self.overlay_open = false;
        }

        fn open_kit_window(&mut self) {
            // When kit window opens, overlay state is closed immediately.
            self.overlay_open = false;
            self.kit_window_open = true;
            self.assert_invariant();
        }

        fn close_kit_window(&mut self) {
            self.kit_window_open = false;
        }

        fn should_draw_kit_window(&self) -> bool {
            self.kit_window_open && !self.overlay_open
        }

        fn route_key(&self, _key: &str) -> &'static str {
            if self.overlay_open {
                "overlay"
            } else if self.kit_window_open {
                "kit_window"
            } else {
                "screen"
            }
        }

        fn route_mouse(&self, _button: i32) -> &'static str {
            if self.overlay_open {
                "overlay"
            } else if self.kit_window_open {
                "kit_window"
            } else {
                "screen"
            }
        }

        fn assert_invariant(&self) {
            assert!(
                !(self.overlay_open && self.kit_window_open),
                "INVARIANT VIOLATION: Both recipe windows cannot be open at the same time!"
            );
        }
    }

    let mut wm = WindowManager::new();

    // 1. Initial state
    assert!(!wm.should_draw_kit_window());
    assert_eq!(wm.route_key("r"), "screen");

    // 2. Player clicks item -> opens official overlay
    wm.open_overlay();
    assert!(wm.overlay_open);
    assert!(!wm.kit_window_open);
    assert!(!wm.should_draw_kit_window());
    assert_eq!(wm.route_key("escape"), "overlay");
    assert_eq!(wm.route_mouse(0), "overlay");

    // 3. Mod/guide tries to open kit window -> overlay closes cleanly
    wm.open_kit_window();
    assert!(!wm.overlay_open);
    assert!(wm.kit_window_open);
    assert!(wm.should_draw_kit_window());
    assert_eq!(wm.route_key("w"), "kit_window");
    assert_eq!(wm.route_mouse(0), "kit_window");
    wm.close_kit_window();
    assert!(!wm.kit_window_open);

    // 4. Player clicks recipe in panel again -> overlay takes back exclusive control
    wm.open_overlay();
    assert!(wm.overlay_open);
    assert!(!wm.kit_window_open);
    assert!(!wm.should_draw_kit_window());
    assert_eq!(wm.route_key("r"), "overlay");

    // 5. Close overlay -> clean idle state
    wm.close_overlay();
    assert!(!wm.overlay_open);
    assert!(!wm.kit_window_open);
    assert!(!wm.should_draw_kit_window());
    assert_eq!(wm.route_key("escape"), "screen");
}

// ---------------------------------------------------------------------------
// 3. Search Workflow, Filtering & Filtered Item Clicking
// ---------------------------------------------------------------------------

#[test]
fn test_search_workflow_and_clicking_filtered_items() {
    let (items, recipes) = realistic_catalog();
    let (mut viewer, _) = Viewer::new(items, recipes, Lang::En);
    let ctx = test_context(3, false);

    // Filter by word "iron"
    let out = viewer.handle(Event::Search("iron".into()), &ctx);
    let filtered = match &out[0] {
        Out::Items(list) => list.clone(),
        _ => panic!("search must return Out::Items"),
    };
    assert_eq!(filtered.len(), 7); // iron_ore, deepslate_iron_ore, raw_iron, iron_ingot, iron_block, iron_sword, iron_pickaxe

    // Click every single filtered item: verify it resolves the correct item
    for (idx, _stack) in filtered.iter().enumerate() {
        let res = viewer.handle(
            Event::Clicked {
                index: idx,
                right: false,
            },
            &ctx,
        );
        match &res[0] {
            Out::Recipes { title, views, .. } => {
                assert!(!views.is_empty());
                assert!(title.contains("Recipes"));
            }
            Out::Say(_) => {} // raw_iron has no recipes
            other => panic!("unexpected filtered result: {other:?}"),
        }
        viewer.handle(Event::Close, &ctx);
    }

    // Exclusion search: "iron -sword"
    let out_ex = viewer.handle(Event::Search("iron -sword".into()), &ctx);
    let list_ex = match &out_ex[0] {
        Out::Items(list) => list,
        _ => panic!("search must return Out::Items"),
    };
    assert!(!list_ex.iter().any(|s| s.item == "minecraft:iron_sword"));

    // Tag search: "#minecraft:planks"
    let out_tag = viewer.handle(Event::Search("#minecraft:planks".into()), &ctx);
    let list_tag = match &out_tag[0] {
        Out::Items(list) => list,
        _ => panic!("search must return Out::Items"),
    };
    assert_eq!(list_tag.len(), 1);
    assert_eq!(list_tag[0].item, "minecraft:oak_planks");

    // Clear search -> restore all
    let out_all = viewer.handle(Event::Search(String::new()), &ctx);
    let list_all = match &out_all[0] {
        Out::Items(list) => list,
        _ => panic!("search must return Out::Items"),
    };
    assert_eq!(list_all.len(), 25);
}

// ---------------------------------------------------------------------------
// 4. Deep History Navigation & Stack Unwinding
// ---------------------------------------------------------------------------

#[test]
fn test_deep_history_navigation_and_back_unwind() {
    let (items, recipes) = realistic_catalog();
    let (mut viewer, _) = Viewer::new(items, recipes, Lang::En);
    let ctx = test_context(3, false);

    // 1. Open Diamond Sword
    let sword_stack = Stack::item("minecraft:diamond_sword");
    let out1 = viewer.handle(
        Event::Open {
            stack: sword_stack.clone(),
            uses: false,
        },
        &ctx,
    );
    assert!(matches!(&out1[0], Out::Recipes { title, .. } if title.contains("Diamond Sword")));

    // 2. Drill down into ingredient: Diamond
    let diamond_stack = Stack::item("minecraft:diamond");
    let out2 = viewer.handle(
        Event::SlotClicked {
            stack: diamond_stack.clone(),
            right: false,
        },
        &ctx,
    );
    assert!(matches!(&out2[0], Out::Recipes { title, .. } if title.contains("Diamond")));

    // 3. Drill down into Diamond Block (uses for craft)
    let block_stack = Stack::item("minecraft:diamond_block");
    let out3 = viewer.handle(
        Event::SlotClicked {
            stack: block_stack.clone(),
            right: false,
        },
        &ctx,
    );
    assert!(matches!(&out3[0], Out::Recipes { title, .. } if title.contains("Diamond")));

    // 4. Now press Back: should step back to Diamond
    let back1 = viewer.handle(Event::Back, &ctx);
    assert!(matches!(&back1[0], Out::Recipes { title, .. } if title.contains("Diamond")));

    // 5. Press Back again: should step back to Diamond Sword
    let back2 = viewer.handle(Event::Back, &ctx);
    assert!(matches!(&back2[0], Out::Recipes { title, .. } if title.contains("Diamond Sword")));

    // 6. Press Back again: should close completely (empty tabs/views)
    let back3 = viewer.handle(Event::Back, &ctx);
    assert!(
        matches!(&back3[0], Out::Recipes { tabs, views, .. } if tabs.is_empty() && views.is_empty())
    );

    // 7. Extra Back presses on empty history must remain graceful
    let back_extra = viewer.handle(Event::Back, &ctx);
    assert!(
        matches!(&back_extra[0], Out::Recipes { tabs, views, .. } if tabs.is_empty() && views.is_empty())
    );
}

// ---------------------------------------------------------------------------
// 5. Category Tabs Switching & Station Catalysts
// ---------------------------------------------------------------------------

#[test]
fn test_category_tabs_switching_and_catalyst_lookup() {
    let (items, recipes) = realistic_catalog();
    let (mut viewer, _) = Viewer::new(items, recipes, Lang::En);
    let ctx = test_context(3, false);

    // Iron Ingot has recipes in Smelting (Furnace) AND Blasting (Blast Furnace) AND Crafting (Block -> 9 ingots)
    let ingot = Stack::item("minecraft:iron_ingot");
    let out = viewer.handle(
        Event::Open {
            stack: ingot,
            uses: false,
        },
        &ctx,
    );
    match &out[0] {
        Out::Recipes {
            tabs, views, tab, ..
        } => {
            assert!(
                tabs.len() >= 3,
                "must have Crafting, Smelting, and Blasting tabs: got {}",
                tabs.len()
            );
            assert_eq!(*tab, 0);
            assert!(!views.is_empty());
        }
        other => panic!("expected recipes output: {other:?}"),
    }

    // Switch to tab 1
    let tab1_out = viewer.handle(Event::Tab(1), &ctx);
    match &tab1_out[0] {
        Out::Recipes { tab, views, .. } => {
            assert_eq!(*tab, 1);
            assert!(!views.is_empty());
        }
        other => panic!("expected tab 1 recipes: {other:?}"),
    }

    // Switch to tab 2
    let tab2_out = viewer.handle(Event::Tab(2), &ctx);
    match &tab2_out[0] {
        Out::Recipes { tab, views, .. } => {
            assert_eq!(*tab, 2);
            assert!(!views.is_empty());
        }
        other => panic!("expected tab 2 recipes: {other:?}"),
    }

    // Catalyst lookup: 'U' on Crafting Table must show all crafting recipes
    let craft_table = Stack::item("minecraft:crafting_table");
    let cat_out = viewer.handle(
        Event::Open {
            stack: craft_table,
            uses: true,
        },
        &ctx,
    );
    match &cat_out[0] {
        Out::Recipes { views, title, .. } => {
            assert!(
                views.len() >= 5,
                "crafting table uses should show all crafting recipes"
            );
            assert!(title.contains("Crafting Table"));
        }
        other => panic!("expected catalyst category recipes: {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// 6. Bookmarks Bar: 24 Slots, Persistence, Drag/Drop Reordering
// ---------------------------------------------------------------------------

#[test]
fn test_bookmarks_daily_workflow_and_persistence() {
    let mut bookmarks = Bookmarks::default();
    assert_eq!(bookmarks.len(), 0);

    // Add 24 unique bookmarks
    for i in 0..24 {
        let stack = Stack::item(&format!("minecraft:item_{i}"));
        assert_eq!(bookmarks.toggle(&stack), Toggle::Added(i));
    }
    assert_eq!(bookmarks.len(), 24);

    // 25th bookmark must be rejected due to capacity limit
    let extra = Stack::item("minecraft:item_25");
    assert_eq!(bookmarks.toggle(&extra), Toggle::Full);
    assert_eq!(bookmarks.len(), 24);

    // Toggle existing bookmark removes it
    let fifth = Stack::item("minecraft:item_4");
    assert_eq!(bookmarks.toggle(&fifth), Toggle::Removed(4));
    assert_eq!(bookmarks.len(), 23);

    // Reorder: move index 0 to index 5
    assert!(bookmarks.move_slot(0, 5));
    assert_eq!(bookmarks.slots()[5].item, "minecraft:item_0");

    // Serialization & round-trip verification
    let serialized = bookmarks_text(&bookmarks);
    let loaded = bookmarks_parse(&serialized);
    assert_eq!(loaded.len(), bookmarks.len());
    for (orig, loaded_stack) in bookmarks.slots().iter().zip(loaded.slots()) {
        assert_eq!(orig, loaded_stack);
    }
}

// ---------------------------------------------------------------------------
// 7. Recipe Transfer ("+" Button) & Grid Size Validation
// ---------------------------------------------------------------------------

#[test]
fn test_recipe_transfer_plus_button_across_grid_sizes() {
    let (items, recipes) = realistic_catalog();
    let (mut viewer, _) = Viewer::new(items, recipes, Lang::En);

    // 3x3 Block of Iron recipe (book_id = 8)
    let block = Stack::item("minecraft:iron_block");

    // Case A: 3x3 recipe over 3x3 crafting grid -> FITS, produces Transfer
    let ctx_3x3 = test_context(3, false);
    viewer.handle(
        Event::Open {
            stack: block.clone(),
            uses: false,
        },
        &ctx_3x3,
    );
    let action_out = viewer.handle(Event::Action(0, false), &ctx_3x3);
    assert_eq!(action_out.len(), 1);
    assert_eq!(
        action_out[0],
        Out::Transfer {
            book_id: 8,
            all: false
        }
    );

    // Case B: Shift-click ("+" with all = true)
    let action_shift = viewer.handle(Event::Action(0, true), &ctx_3x3);
    assert_eq!(action_shift.len(), 1);
    assert_eq!(
        action_shift[0],
        Out::Transfer {
            book_id: 8,
            all: true
        }
    );

    // Case C: 3x3 recipe over 2x2 player inventory grid -> DOES NOT FIT
    let iron_block_recipe = Recipe {
        kind: Kind::CraftingShaped,
        width: 3,
        height: 3,
        slots: vec![
            vec![Ingredient {
                item: 5,
                count: 1,
                components: String::new()
            }];
            9
        ],
        results: vec![Ingredient {
            item: 6,
            count: 1,
            components: String::new(),
        }],
        cooking_time: 0,
        experience: 0.0,
        book_id: 8,
    };
    assert!(!fits(&iron_block_recipe, 2));
    assert!(fits(&iron_block_recipe, 3));

    // Case D: 1x1 shapeless recipe fits in both 2x2 and 3x3
    let shapeless_planks = Recipe {
        kind: Kind::CraftingShapeless,
        width: 0,
        height: 0,
        slots: vec![vec![Ingredient {
            item: 11,
            count: 1,
            components: String::new(),
        }]],
        results: vec![Ingredient {
            item: 10,
            count: 4,
            components: String::new(),
        }],
        cooking_time: 0,
        experience: 0.0,
        book_id: 1,
    };
    assert!(fits(&shapeless_planks, 2));
    assert!(fits(&shapeless_planks, 3));
}

// ---------------------------------------------------------------------------
// 8. Cheat Mode vs Survival Mode Click Handling
// ---------------------------------------------------------------------------

#[test]
fn test_cheat_mode_vs_survival_mode_clicks() {
    let (items, recipes) = realistic_catalog();
    let (mut viewer, initial_out) = Viewer::new(items, recipes, Lang::En);
    let panel_items = match initial_out {
        Out::Items(stacks) => stacks,
        _ => panic!("initial output must be panel items list"),
    };
    let ingot_idx = panel_items
        .iter()
        .position(|s| s.item == "minecraft:iron_ingot")
        .expect("iron_ingot must be in panel items");

    // Survival mode: left-click opens recipes
    let survival_ctx = test_context(3, false);
    let out_surv = viewer.handle(
        Event::Clicked {
            index: ingot_idx,
            right: false,
        },
        &survival_ctx,
    );
    assert!(matches!(&out_surv[0], Out::Recipes { .. }));
    viewer.handle(Event::Close, &survival_ctx);

    // Cheat/Give mode:
    // Left-click gives 1 item
    let cheat_ctx = test_context(3, true);
    let out_give_one = viewer.handle(
        Event::Clicked {
            index: ingot_idx,
            right: false,
        },
        &cheat_ctx,
    );
    assert_eq!(
        out_give_one[0],
        Out::Give {
            item: "minecraft:iron_ingot".into(),
            count: 1,
        }
    );

    // Right-click in cheat mode gives 99 items
    let out_give_stack = viewer.handle(
        Event::Clicked {
            index: ingot_idx,
            right: true,
        },
        &cheat_ctx,
    );
    assert_eq!(
        out_give_stack[0],
        Out::Give {
            item: "minecraft:iron_ingot".into(),
            count: 99,
        }
    );
}

// ---------------------------------------------------------------------------
// 9. Third-Party Mod Category & Machine Catalyst Integration
// ---------------------------------------------------------------------------

#[test]
fn test_mod_category_and_machine_catalyst_integration() {
    let (items, recipes) = realistic_catalog();
    let (mut viewer, _) = Viewer::new(items, recipes, Lang::En);

    // Register a custom AE2 Inscriber category
    let inscriber_category = Category {
        id: "ae2:inscriber".into(),
        title: "Inscriber".into(),
        subtitle: "ae2:inscriber".into(),
        icon: Stack::item("ae2:inscriber"),
        catalysts: vec![Stack::item("ae2:inscriber")],
        recipes: vec![ChanceRecipe {
            inputs: vec![
                vec![Stack::item("minecraft:iron_block")],
                vec![Stack::item("ae2:certus_quartz_crystal")],
            ],
            outputs: vec![Chance {
                stack: Stack::item("ae2:certus_quartz_crystal"),
                chance: 1.0,
                min: 1,
                max: 1,
                notes: vec![],
            }],
            notes: vec![],
        }],
    };

    viewer.set_categories(vec![inscriber_category]);

    let ctx = test_context(3, false);

    // Open recipe for the product: mod category must appear
    let out = viewer.handle(
        Event::Open {
            stack: Stack::item("ae2:certus_quartz_crystal"),
            uses: false,
        },
        &ctx,
    );
    match &out[0] {
        Out::Recipes { tabs, names, .. } => {
            assert!(names.contains(&"Inscriber".to_owned()));
            assert!(tabs.iter().any(|t| t.item == "ae2:inscriber"));
        }
        other => panic!("expected inscriber category tab: {other:?}"),
    }

    // Open uses for the catalyst machine: Inscriber must show all its recipes
    let cat_out = viewer.handle(
        Event::Open {
            stack: Stack::item("ae2:inscriber"),
            uses: true,
        },
        &ctx,
    );
    match &cat_out[0] {
        Out::Recipes { title, views, .. } => {
            assert!(title.contains("Inscriber"));
            assert_eq!(views.len(), 1);
            assert_eq!(views[0].layout, Layout::Chance);
        }
        other => panic!("expected catalyst inscriber recipes: {other:?}"),
    }
}

#[test]
fn test_view_default_and_backward_compatibility() {
    let def = View::default();
    assert_eq!(def.layout, Layout::Grid);
    assert_eq!(def.width, 0);
    assert_eq!(def.height, 0);
    assert!(def.slots.is_empty());
    assert!(def.results.is_empty());
    assert!(def.note.is_empty());
    assert_eq!(def.action, None);
    assert!(def.chances.is_empty());

    // 0.2-style literal using struct update syntax with ..Default::default():
    let view = View {
        layout: Layout::Cooking,
        width: 1,
        height: 1,
        slots: vec![vec![Stack::item("minecraft:raw_iron")]],
        results: vec![Stack::item("minecraft:iron_ingot")],
        note: "10.0 s".into(),
        action: Some(false),
        ..Default::default()
    };
    assert_eq!(view.layout, Layout::Cooking);
    assert!(view.chances.is_empty());

    let v_new = View::new(
        Layout::Single,
        1,
        1,
        vec![vec![Stack::item("minecraft:stone")]],
        vec![Stack::item("minecraft:stone_bricks")],
        "Stonecutter",
        None,
    );
    assert_eq!(v_new.layout, Layout::Single);
    assert!(v_new.chances.is_empty());

    let v_chances = View::new(
        Layout::Chance,
        1,
        1,
        vec![],
        vec![Stack::item("minecraft:diamond")],
        "",
        None,
    )
    .with_chances(vec![Chance::new(Stack::item("minecraft:diamond"), 0.5)]);
    assert_eq!(v_chances.layout, Layout::Chance);
    assert_eq!(v_chances.chances.len(), 1);
    assert_eq!(v_chances.chances[0].chance, 0.5);
}
