use std::collections::VecDeque;

use crate::shared::*;

/// Build the new-game intro cutscene step queue.
///
/// Sequence:
/// 1. FadeOut(0.0) — start on black
/// 2. ShowText — grandfather's letter
/// 3. ShowText — the letter contents
/// 4. ShowText — day card
/// 5. ShowText — controls card (shown early, while still on black)
/// 6. FadeIn — reveal the farm
/// 7. Wait — let player see the farm
/// 8. StartDialogueCustom — Mayor Rex greets the player
/// 9. WaitForDialogueEnd
pub fn build_intro_sequence() -> VecDeque<CutsceneStep> {
    let mut steps = VecDeque::new();

    // Start on black (instant).
    steps.push_back(CutsceneStep::FadeOut(0.0));

    // Narrative text cards.
    steps.push_back(CutsceneStep::ShowText(
        "Three years ago, your grandfather left you a letter...".into(),
        4.0,
    ));
    steps.push_back(CutsceneStep::ShowText(
        "'Dear child, I've left you Hearthfield Farm.\nIt's not much, but it's honest land.\nMake something of it.'".into(),
        5.0,
    ));

    // Day card.
    steps.push_back(CutsceneStep::ShowText("Spring 1, Year 1".into(), 2.5));

    // Controls reference card — shown early so the player reads controls
    // while the screen is still black, before they need to do anything.
    steps.push_back(CutsceneStep::ShowText(
        "Controls: WASD = Move | Space = Use Tool | F = Interact\n\
         R = Eat Food / Place Item | E = Inventory | C = Crafting\n\
         [ ] = Cycle Tools | 1-9 = Select Hotbar | Esc = Pause"
            .into(),
        8.0,
    ));

    // Signal NPC spawning to place Mayor Rex on the farm for the intro greeting.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), true));

    // Reveal the farm.
    steps.push_back(CutsceneStep::FadeIn(1.5));

    // Let the player see the farm for a moment.
    steps.push_back(CutsceneStep::Wait(1.0));

    // Mayor Rex greets the player with intro-specific lines.
    steps.push_back(CutsceneStep::StartDialogueCustom {
        npc_id: "mayor_rex".into(),
        lines: vec![
            "Ah, you must be the new farmer! Welcome to Hearthfield.".into(),
            "Your grandfather was a good man. This farm meant everything to him.".into(),
            "You'll find seeds in your pack to get started. Till some soil with your hoe, then plant them.".into(),
            "Come visit me in town if you need anything. And don't forget to sleep before midnight!".into(),
        ],
        portrait_index: Some(7),
    });
    steps.push_back(CutsceneStep::WaitForDialogueEnd);

    // Tool tutorial — Mayor Rex walks the player through each tool.
    // Set the flag so the overlay system knows to show tool sprites.
    steps.push_back(CutsceneStep::SetFlag("tool_tutorial_active".into(), true));
    steps.push_back(CutsceneStep::StartDialogueCustom {
        npc_id: "mayor_rex".into(),
        lines: vec![
            // Line 0 — intro (no overlay)
            "Let me show you your tools! Every farmer needs to know their equipment.".into(),
            // Line 1 — HOE overlay shown
            "This is your HOE. Till the soil with SPACE, then plant seeds.".into(),
            // Line 2 — WATERING CAN overlay
            "The WATERING CAN keeps crops alive. Water them every day!".into(),
            // Line 3 — AXE overlay
            "Your AXE chops trees for wood. You'll need lumber for upgrades.".into(),
            // Line 4 — PICKAXE overlay
            "The PICKAXE breaks rocks in the mines. Find ores and gems!".into(),
            // Line 5 — SCYTHE overlay
            "The SCYTHE cuts grass and harvests ripe crops. Fast and efficient!".into(),
            // Line 6 — FISHING ROD overlay
            "And this FISHING ROD — cast into water, wait for a bite, then reel in!".into(),
            // Line 7 — outro (no overlay)
            "That's everything! Use [ ] to cycle tools, or 1-6 for quick select. Good luck!".into(),
        ],
        portrait_index: Some(7),
    });
    steps.push_back(CutsceneStep::WaitForDialogueEnd);
    // Clear the tutorial flag now that it's been shown.
    steps.push_back(CutsceneStep::SetFlag("tool_tutorial_active".into(), false));

    // Clean up — Mayor Rex returns to his scheduled location.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), false));

    steps
}
