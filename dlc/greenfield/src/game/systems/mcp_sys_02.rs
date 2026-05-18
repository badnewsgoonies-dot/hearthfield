use bevy::prelude::*;
use crate::game::components::McpStruct02;

/// MCP scale variant 02 — spawns a McpStruct02 entity on first run,
/// then no-ops. Wired into McpPlugin02::build() at v15.
pub fn mcp_sys_02_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct02::new_02());
    *spawned = true;
}
