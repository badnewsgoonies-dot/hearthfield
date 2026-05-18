use bevy::prelude::*;
use crate::game::components::McpStruct04;

/// MCP scale variant 04 — spawns a McpStruct04 entity on first run,
/// then no-ops. Wired into McpPlugin04::build() at v15.
pub fn mcp_sys_04_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct04::new_04());
    *spawned = true;
}
