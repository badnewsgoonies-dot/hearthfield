use bevy::prelude::*;
use crate::game::components::McpStruct07;

/// MCP scale variant 07 — spawns a McpStruct07 entity on first run,
/// then no-ops. Wired into McpPlugin07::build() at v15.
pub fn mcp_sys_07_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct07::new_07());
    *spawned = true;
}
