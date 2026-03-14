use std::collections::VecDeque;

use crate::shared::*;

/// Build the new-game intro cutscene step queue.
///
/// Sequence:
/// 1. FadeOut(0.0) — start on black
/// 2. SetFlag — place Mayor Rex for the intro reveal
/// 3. ShowText — day card
/// 4. ShowText — wake-up prompt
/// 5. ShowText — first-morning card
/// 6. ShowText — step-outside prompt
/// 7. FadeIn — reveal the farm
/// 8. Wait — let player see the farm
/// 9. StartDialogueCustom — Mayor Rex greets the player
/// 10. WaitForDialogueEnd
/// 11. SetFlag(false) — return Mayor Rex to his schedule
pub fn build_intro_sequence() -> VecDeque<CutsceneStep> {
    let mut steps = VecDeque::new();

    // Start on black (instant).
    steps.push_back(CutsceneStep::FadeOut(0.0));

    // Signal NPC spawning to place Mayor Rex on the farm before the reveal.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), true));

    // Narrative text cards.
    steps.push_back(CutsceneStep::ShowText(
        "Spring 1, Year 1".into(),
        0.6,
    ));
    steps.push_back(CutsceneStep::ShowText(
        "Mayor Rex is waiting outside the farmhouse door.".into(),
        0.7,
    ));

    // First-morning card.
    steps.push_back(CutsceneStep::ShowText(
        "This is your first morning as the new farmer.".into(),
        0.8,
    ));

    // Keep the final setup prompt brief so the intro stays moving.
    steps.push_back(CutsceneStep::ShowText(
        "Step outside and meet him.".into(),
        0.6,
    ));

    // Reveal the farm.
    steps.push_back(CutsceneStep::FadeIn(0.6));

    // Let the player see the farm for a moment.
    steps.push_back(CutsceneStep::Wait(0.9));

    // Mayor Rex greets the player with intro-specific lines.
    steps.push_back(CutsceneStep::StartDialogueCustom {
        npc_id: "mayor_rex".into(),
        lines: vec![
            "Morning. I'm Mayor Rex. I wanted to be here when you stepped into your new life.".into(),
            "Your turnip seeds are already in your pack. Plant a few by the house and you'll feel the farm answer back.".into(),
            "When you're ready, take the south path into town. Folks are already wondering who came home to the old farm.".into(),
        ],
        portrait_index: Some(7),
    });
    steps.push_back(CutsceneStep::WaitForDialogueEnd);

    // Clean up — Mayor Rex returns to his scheduled location.
    steps.push_back(CutsceneStep::SetFlag("mayor_intro_visit".into(), false));

    steps
}
