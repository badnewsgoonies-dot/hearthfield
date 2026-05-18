use bevy::prelude::*;
use crate::game::components::McpStruct05;

/// MCP scale variant 05 — spawns a McpStruct05 entity on first run,
/// then no-ops. Wired into McpPlugin05::build() at v15.
pub fn mcp_sys_05_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct05::new_05());
    *spawned = true;
}
