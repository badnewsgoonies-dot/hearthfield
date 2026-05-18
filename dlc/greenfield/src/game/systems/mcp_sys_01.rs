use bevy::prelude::*;
use crate::game::components::McpStruct01;

/// MCP scale variant 01 — spawns a McpStruct01 entity on first run,
/// then no-ops. Wired into McpPlugin01::build() at v15.
pub fn mcp_sys_01_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct01::new_01());
    *spawned = true;
}
