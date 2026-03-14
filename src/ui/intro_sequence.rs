use std::collections::VecDeque;

use crate::shared::*;

/// Build the new-game intro cutscene step queue.
///
/// Sequence:
/// 1. FadeOut(0.0) — start on black
/// 2. ShowText — grandfather's letter
/// 3. ShowText — the letter contents
/// 4. ShowText — day card
/// 5. ShowText — quick controls card (shown early, while still on black)
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
        "Your grandfather left you one last letter...".into(),
        3.0,
    ));
    steps.push_back(CutsceneStep::ShowText(
        "'Dear child,\nWelcome to your new life at Hearthfield Farm.\nTake this little patch of land and make it your own.'".into(),
        4.5,
    ));

    // Day card.
    steps.push_back(CutsceneStep::ShowText("Spring 1, Year 1".into(), 2.5));

    // Controls reference card — kept brief so the intro stays moving.
    steps.push_back(CutsceneStep::ShowText(
        "Move: WASD | Use tool: Space | Interact: F\n\
         Inventory: E | Crafting: C | Cycle tools: [ ]"
            .into(),
        4.0,
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
            "There you are. Welcome to Hearthfield, and welcome to your new life.".into(),
            "Your grandfather cared deeply for this place. I think he'd be glad to see you here.".into(),
            "I've tucked a few seeds into your pack to help you start.".into(),
            "Try tilling the soil outside with your hoe, then plant those seeds. Come find me in town if you need a hand.".into(),
        ],
        portrait_index: Some(7),
    });
    steps.push_back(CutsceneStep::WaitForDialogueEnd);

    // Clean up — Mayor Rex returns to his scheduled location.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), false));

    steps
}
