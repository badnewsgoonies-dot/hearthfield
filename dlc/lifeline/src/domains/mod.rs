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


pub mod ported_bed_tile_hash;

pub mod ported_score_to_credits;

pub mod ported_set_pixel_helper;

pub mod ported_set_px_helper;

pub mod ported_shift_name_for_date;

pub mod ported_ward_image_source;

pub mod ported_facing_delta_helper;

pub mod ported_tint_hash;

pub mod ported_format_credits;

pub mod ported_compact_med_name;

pub mod ported_icon_size_node_helper;

pub mod ported_simple_hash_helper;

pub mod ported_patient_atlas_index;

pub mod ported_trust_tier;

pub mod ported_rare_case_display_name;

pub mod ported_rare_case_sell_price;

pub mod ported_preference_to_points_helper;

pub mod ported_tag_color_helper;

pub mod ported_ward_decor_png_size;

pub mod ported_ward_object_atlas_index;

pub mod ported_object_tint_helper;

pub mod ported_put_pixel_helper;
