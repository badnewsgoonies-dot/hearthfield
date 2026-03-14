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
        1.6,
    ));
    steps.push_back(CutsceneStep::ShowText(
        "'Dear child,\nWelcome to your new life at Hearthfield Farm.\nTake this little patch of land and make it your own.'".into(),
        2.4,
    ));

    // Day card.
    steps.push_back(CutsceneStep::ShowText("Spring 1, Year 1".into(), 1.2));

    // Controls reference card — kept brief so the intro stays moving.
    steps.push_back(CutsceneStep::ShowText(
        "Step outside. Till, plant, and water your turnip seeds.".into(),
        1.8,
    ));

    // Signal NPC spawning to place Mayor Rex on the farm for the intro greeting.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), true));

    // Reveal the farm.
    steps.push_back(CutsceneStep::FadeIn(0.9));

    // Let the player see the farm for a moment.
    steps.push_back(CutsceneStep::Wait(1.0));

    // Mayor Rex greets the player with intro-specific lines.
    steps.push_back(CutsceneStep::StartDialogueCustom {
        npc_id: "mayor_rex".into(),
        lines: vec![
            "There you are. Welcome home.".into(),
            "Hearthfield's been waiting for someone to love it again.".into(),
            "Your turnip seeds are already in your pack. Start with one patch by the house: till it, plant it, water it.".into(),
            "After that, take the south path into town whenever you're ready.".into(),
        ],
        portrait_index: Some(7),
    });
    steps.push_back(CutsceneStep::WaitForDialogueEnd);

    // Clean up — Mayor Rex returns to his scheduled location.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), false));

    steps
}
