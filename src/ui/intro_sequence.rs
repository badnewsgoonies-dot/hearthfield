use std::collections::VecDeque;

use crate::shared::*;

/// Build the new-game intro cutscene step queue.
///
/// Sequence:
/// 1. FadeOut(0.0) — start on black
/// 2. ShowText — grandfather's letter
/// 3. ShowText — the letter contents
/// 4. ShowText — day card
/// 5. FadeIn — reveal the farm
/// 6. Wait — let player see the farm
/// 7. StartDialogueCustom — Mayor Rex greets the player
/// 8. WaitForDialogueEnd
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
    steps.push_back(CutsceneStep::ShowText(
        "Spring 1, Year 1".into(),
        2.5,
    ));

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

    steps
}
