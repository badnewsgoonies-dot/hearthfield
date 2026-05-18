use bevy::prelude::*;
use crate::game::components::McpStruct06;

/// MCP scale variant 06 — spawns a McpStruct06 entity on first run,
/// then no-ops. Wired into McpPlugin06::build() at v15.
pub fn mcp_sys_06_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct06::new_06());
    *spawned = true;
}
