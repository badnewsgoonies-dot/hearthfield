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

pub mod ported_ward_label;

pub mod ported_facing_offset_player;

pub mod ported_facing_offset_chest;

pub mod ported_facing_rotation_sign_helper;

pub mod ported_shift_display_name;

pub mod ported_shift_for_cycle;

pub mod ported_decor_color;

pub mod ported_bed_color;

pub mod ported_tier_label_helper;

pub mod ported_ward_object_color;

pub mod ported_tool_damage;

pub mod ported_tool_fatigue_cost;

pub mod ported_staff_action_damage;

pub mod ported_severity_text_color;

pub mod ported_severity_weight;

pub mod ported_tier_level_helper;

pub mod ported_map_to_case_location;

pub mod ported_bed_atlas_index;

pub mod ported_tile_palette;

pub mod ported_autotile_idx;

pub mod ported_atlas_idx;

pub mod ported_legend_price;

pub mod ported_set_pixel;

pub mod ported_autotile_idx2;

pub mod ported_fmt_gold;

pub mod ported_pts_to_candles;

pub mod ported_shimmer_color;

pub mod ported_format_gold_alt;

pub mod ported_read_dpad_just_pressed_helper;

pub mod ported_tool_hand_offset_helper;

pub mod ported_tool_tier_label_helper;

pub mod ported_world_to_grid_helper;

pub mod ported_xp_for_rarity_helper;

pub mod ported_square_area_helper;

pub mod ported_stamina_cost_helper;

pub mod ported_terrain_tint_helper;

pub mod ported_tool_display_name_helper;

pub mod ported_weekday_name_helper;

pub mod ported_summarize_next_festival_helper;

pub mod ported_tool_kind_from_item_id_helper;

pub mod ported_buff_type_label_helper;

pub mod ported_bush_variant_color_helper;

pub mod ported_happiness_icon_index_helper;

pub mod ported_shimmer_color_for_ore_helper;

pub mod ported_tool_frame_duration_helper;

pub mod ported_candle_positions_helper;

pub mod ported_is_indoor_map_helper;

pub mod ported_rock_atlas_index_helper;

pub mod ported_format_gold_helper;

pub mod ported_is_outdoor_map_helper;

pub mod ported_crop_stage_color_helper;

pub mod ported_machine_atlas_index_helper;

pub mod ported_npc_sprite_file_helper;

pub mod ported_npc_color_helper;

pub mod ported_read_dpad_axis_helper;

pub mod ported_item_to_machine_type_helper;

pub mod ported_preference_toast_message_helper;

pub mod ported_quality_from_happiness_helper;

pub mod ported_is_path_neighbor_helper;

pub mod ported_music_path_helper;

pub mod ported_write_all_ron_files_helper;

pub mod ported_bait_bite_multiplier_helper;

pub mod ported_day_tag_helper;

pub mod ported_path_autotile_index_helper;

pub mod ported_format_last_saved_helper;

pub mod ported_map_display_name_helper;

pub mod ported_map_id_display_name_helper;

pub mod ported_sprinkler_affected_tiles_helper;

pub mod ported_map_bounds_hardcoded_helper;

pub mod ported_map_id_filename_helper;

pub mod ported_tree_tint_helper;

pub mod ported_decorative_fence_mask_helper;

pub mod ported_default_spawn_position_helper;

pub mod ported_fallback_tile_palette_helper;
