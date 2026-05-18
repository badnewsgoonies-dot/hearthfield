use bevy::prelude::*;
use crate::game::components::McpStruct09;

/// MCP scale variant 09 — spawns a McpStruct09 entity on first run,
/// then no-ops. Wired into McpPlugin09::build() at v15.
pub fn mcp_sys_09_system(mut commands: Commands, mut spawned: Local<bool>) {
    if *spawned { return; }
    commands.spawn(McpStruct09::new_09());
    *spawned = true;
}
