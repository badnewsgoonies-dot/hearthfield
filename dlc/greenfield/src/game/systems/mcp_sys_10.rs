use bevy::prelude::*;
use crate::game::components::McpStruct10;

/// MCP scale variant 10 — spawns a McpStruct10 entity on first run,
/// then no-ops. Wired into McpPlugin10::build() at v15.
pub fn mcp_sys_10_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct10::new_10());
    *spawned = true;
}
