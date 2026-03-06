//! Regression test: no unintended duplicate keybindings.
use bevy::prelude::KeyCode;
use hearthfield::shared::KeyBindings;
use std::collections::HashMap;

#[test]
fn default_keybindings_have_no_unintended_duplicates() {
    let b = KeyBindings::default();
    let mut map: HashMap<KeyCode, Vec<&'static str>> = HashMap::new();

    macro_rules! add {
        ($key:expr, $name:expr) => {
            map.entry($key).or_default().push($name);
        };
    }

    add!(b.move_up, "move_up");
    add!(b.move_down, "move_down");
    add!(b.move_left, "move_left");
    add!(b.move_right, "move_right");
    add!(b.interact, "interact");
    add!(b.tool_use, "tool_use");
    add!(b.tool_secondary, "tool_secondary");
    add!(b.open_inventory, "open_inventory");
    add!(b.open_crafting, "open_crafting");
    add!(b.open_map, "open_map");
    add!(b.open_journal, "open_journal");
    add!(b.open_relationships, "open_relationships");
    add!(b.pause, "pause");
    add!(b.tool_next, "tool_next");
    add!(b.tool_prev, "tool_prev");
    add!(b.ui_confirm, "ui_confirm");
    add!(b.ui_cancel, "ui_cancel");
    add!(b.skip_cutscene, "skip_cutscene");

    // These are intentionally shared across input contexts:
    let allow = [KeyCode::Escape, KeyCode::Space];

    for (k, fields) in &map {
        if fields.len() > 1 && !allow.contains(k) {
            panic!("Duplicate keybinding: {:?} is used by {:?}", k, fields);
        }
    }
}
