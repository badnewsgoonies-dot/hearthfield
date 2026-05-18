//! Lifeline domain roster. Each domain is an isolated Bevy plugin.
//! Briefcase transforms splice events / components / systems into the
//! individual domain modules; this file just wires them into the app.

pub mod calendar;
pub mod diagnostics;
pub mod economy;
pub mod npcs;
pub mod patients;
pub mod pharmacy;
pub mod player;
pub mod ported;
pub mod rounds;
pub mod save;
pub mod skills;
pub mod ui;
pub mod world;


pub mod _debug_solo;

pub mod ported_consume;

pub mod ported_refund;

pub mod ported_admit;
