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



pub mod ported_consume;

pub mod ported_refund;

pub mod ported_admit;


pub mod ported_shift_overlay_cleanup;

pub mod ported_ledger_cleanup;

pub mod ported_kit_screen_cleanup;

pub mod ported_chart_screen_cleanup;

pub mod ported_staff_screen_cleanup;

pub mod ported_treatment_screen_cleanup;

pub mod ported_ward_upgrade_cleanup;

pub mod ported_touch_overlay_cleanup;

pub mod ported_clamp_helper;

pub mod ported_shift_for_date;

pub mod ported_format_shift_time;
