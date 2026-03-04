use bevy::prelude::*;

use crate::game::events::{
    CoffeeBreakEvent, CoworkerHelpEvent, InterruptionEvent, ManagerCheckInEvent,
    PanicResponseEvent, ProcessInboxEvent, ResolveCalmlyEvent, WaitEvent,
};
use crate::game::resources::OfficeRules;

#[allow(clippy::too_many_arguments)]
pub fn collect_player_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    rules: Res<OfficeRules>,
    mut process_writer: EventWriter<ProcessInboxEvent>,
    mut coffee_writer: EventWriter<CoffeeBreakEvent>,
    mut interruption_writer: EventWriter<InterruptionEvent>,
    mut calm_writer: EventWriter<ResolveCalmlyEvent>,
    mut panic_writer: EventWriter<PanicResponseEvent>,
    mut manager_writer: EventWriter<ManagerCheckInEvent>,
    mut coworker_writer: EventWriter<CoworkerHelpEvent>,
    mut wait_writer: EventWriter<WaitEvent>,
) {
    if keyboard.just_pressed(KeyCode::KeyP) {
        process_writer.send(ProcessInboxEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyC) {
        coffee_writer.send(CoffeeBreakEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyI) {
        interruption_writer.send(InterruptionEvent {
            kind: crate::game::resources::InterruptionKind::CoworkerHelp,
        });
    }
    if keyboard.just_pressed(KeyCode::Digit1) {
        calm_writer.send(ResolveCalmlyEvent);
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        panic_writer.send(PanicResponseEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyM) {
        manager_writer.send(ManagerCheckInEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyH) {
        coworker_writer.send(CoworkerHelpEvent);
    }
    if keyboard.just_pressed(KeyCode::KeyN) {
        wait_writer.send(WaitEvent {
            minutes: rules.wait_minutes,
        });
    }
}
