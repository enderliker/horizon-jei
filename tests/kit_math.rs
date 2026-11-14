// Unit tests for the kit's arithmetic (mixins/jei/kit_math.rs). kit_math.rs is written by jei-kit/apply.py and
// compiles inside the game as crate::game::hz_kit_math_jei; it uses nothing of the game, so here it compiles on its
// own. In the game its `if_kit_jei_items!` wrapper keeps the newest kit's copy; here there is one copy, so the stand-in
// below keeps the items.

mod jei_optional {
    macro_rules! if_kit_jei_items {
        ($id:literal, { $($items:tt)* }) => { $($items)* };
    }
    pub(crate) use if_kit_jei_items;
}

mod math {
    use crate::jei_optional;
    include!("../mixins/jei/kit_math.rs");
}

use math::*;
use serde_json::json;

const OVERWORLD: (i32, i32) = (-64, 384);

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

#[test]
fn average_of_number_providers() {
    assert!(close(average(&json!(3)), 3.0));
    assert!(close(
        average(&json!({"type": "minecraft:constant", "value": 5})),
        5.0
    ));
    assert!(close(
        average(&json!({"type": "minecraft:uniform", "min": 1, "max": 3})),
        2.0
    ));
    assert!(close(
        average(&json!({"type": "minecraft:binomial", "n": 4, "p": 0.5})),
        2.0
    ));
    // Nested providers, and nothing to read.
    assert!(close(
        average(&json!({"min": {"value": 2}, "max": {"min": 4, "max": 6}})),
        3.5
    ));
    assert!(close(average(&json!({})), 1.0));
}

#[test]
fn range_of_counts() {
    assert_eq!(range(&json!(2)), (2, 2, 1.0));
    assert_eq!(range(&json!(-1)), (0, 0, 1.0));
    assert_eq!(range(&json!({"min": 2, "max": 5})), (2, 5, 1.0));
    // A uniform 0..=2 is 1 or more two times in three.
    let (low, high, chance) = range(&json!({"min": 0, "max": 2}));
    assert_eq!((low, high), (1, 2));
    assert!(close(chance, 2.0 / 3.0));
    // A binomial of 3 tries at 1/2 is above 0 seven times in eight.
    let (low, high, chance) = range(&json!({"n": 3, "p": 0.5}));
    assert_eq!((low, high), (0, 3));
    assert!(close(chance, 0.875));
}

#[test]
fn height_anchors() {
    assert_eq!(anchor(&json!({"absolute": 5}), OVERWORLD), 5);
    assert_eq!(anchor(&json!({"above_bottom": 4}), OVERWORLD), -60);
    assert_eq!(anchor(&json!({"below_top": 0}), OVERWORLD), 319);
    assert_eq!(anchor(&json!({}), OVERWORLD), -64);
}

#[test]
fn height_shapes_sum_to_one() {
    let world = (-64, 8);
    // No height range: every layer alike.
    let flat = height_shape(&[], world);
    assert_eq!(flat.len(), 8);
    assert!(flat.iter().all(|w| close(*w, 1.0 / 8.0)));
    // Uniform over the bottom four layers.
    let uniform = [
        json!({"type": "minecraft:height_range", "height": {"type": "minecraft:uniform",
        "min_inclusive": {"absolute": -64}, "max_inclusive": {"absolute": -61}}}),
    ];
    let shape = height_shape(&uniform, world);
    assert!(shape[..4].iter().all(|w| close(*w, 0.25)));
    assert!(shape[4..].iter().all(|w| *w == 0.0));
    // A trapezoid peaks in its middle and falls off to both sides.
    let trapezoid = [
        json!({"type": "minecraft:height_range", "height": {"type": "minecraft:trapezoid",
        "min_inclusive": {"absolute": -64}, "max_inclusive": {"absolute": -58}}}),
    ];
    let shape = height_shape(&trapezoid, world);
    assert!(close(shape.iter().sum::<f64>(), 1.0));
    assert!(shape[3] > shape[1] && shape[1] > shape[0]);
    assert!(close(shape[2], shape[4]));
}

#[test]
fn tries_per_chunk() {
    assert!(close(tries(&[]), 1.0));
    let placed = [
        json!({"type": "minecraft:count", "count": 8}),
        json!({"type": "minecraft:rarity_filter", "chance": 2}),
    ];
    assert!(close(tries(&placed), 4.0));
    let ranged =
        [json!({"type": "minecraft:count", "count": {"min_inclusive": 2, "max_inclusive": 6}})];
    assert!(close(tries(&ranged), 4.0));
}

#[test]
fn solid_layers_and_veins() {
    assert!(close(solid(1, 200), 1.0));
    assert!(close(solid(0, -64), 0.0));
    assert!(close(solid(0, -59), 1.0));
    assert!(close(solid(0, 300), 0.02));
    assert!(close(vein_blocks(10.0), 6.0));
}

#[test]
fn numbers_and_names_on_the_pages() {
    assert_eq!(percent(1.0), "100%");
    assert_eq!(percent(0.5), "50%");
    assert_eq!(percent(0.05), "5.0%");
    assert_eq!(percent(0.0005), "0.05%");
    assert_eq!(percent(0.00001), "0.001%");
    assert_eq!(trim_count(12.4), "12");
    assert_eq!(trim_count(2.26), "2.3");
    assert_eq!(title_case("iron_ore"), "Iron Ore");
    assert_eq!(title_case("chests/simple_dungeon"), "Chests Simple Dungeon");
}
