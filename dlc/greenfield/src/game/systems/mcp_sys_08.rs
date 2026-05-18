use bevy::prelude::*;
use crate::game::components::McpStruct08;

/// MCP scale variant 08 — spawns a McpStruct08 entity on first run,
/// then no-ops. Wired into McpPlugin08::build() at v15.
pub fn mcp_sys_08_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct08::new_08());
    *spawned = true;
}
