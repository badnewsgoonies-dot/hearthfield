use bevy::prelude::*;
use crate::game::components::McpStruct03;

/// MCP scale variant 03 — spawns a McpStruct03 entity on first run,
/// then no-ops. Wired into McpPlugin03::build() at v15.
pub fn mcp_sys_03_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct03::new_03());
    *spawned = true;
}
