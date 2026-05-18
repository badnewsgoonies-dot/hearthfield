use bevy::prelude::*;
use std::sync::atomic::Ordering;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::AtomicU32;
use std::time::Instant;
use std::time::Duration;

use crate::game::systems::mcp_sys_01::mcp_sys_01_system;
use crate::game::systems::mcp_sys_02::mcp_sys_02_system;
use crate::game::systems::mcp_sys_03::mcp_sys_03_system;
use crate::game::systems::mcp_sys_04::mcp_sys_04_system;
use crate::game::systems::mcp_sys_05::mcp_sys_05_system;
use crate::game::systems::mcp_sys_06::mcp_sys_06_system;
use crate::game::systems::mcp_sys_07::mcp_sys_07_system;
use crate::game::systems::mcp_sys_08::mcp_sys_08_system;
use crate::game::systems::mcp_sys_09::mcp_sys_09_system;
use crate::game::systems::mcp_sys_10::mcp_sys_10_system;

pub struct McpPlugin01;

impl Plugin for McpPlugin01 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_01_system);
    }
}


pub struct McpPlugin02;

impl Plugin for McpPlugin02 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_02_system);
    }
}


pub struct McpPlugin03;

impl Plugin for McpPlugin03 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_03_system);
    }
}


pub struct McpPlugin04;

impl Plugin for McpPlugin04 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_04_system);
    }
}


pub struct McpPlugin05;

impl Plugin for McpPlugin05 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_05_system);
    }
}


pub struct McpPlugin06;

impl Plugin for McpPlugin06 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_06_system);
    }
}


pub struct McpPlugin07;

impl Plugin for McpPlugin07 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_07_system);
    }
}


pub struct McpPlugin08;

impl Plugin for McpPlugin08 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_08_system);
    }
}


pub struct McpPlugin09;

impl Plugin for McpPlugin09 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_09_system);
    }
}


pub struct McpPlugin10;

impl Plugin for McpPlugin10 {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, mcp_sys_10_system);
    }
}


impl McpPlugin01 {}

impl McpPlugin02 {}

impl McpPlugin03 {}

impl McpPlugin04 {}

impl McpPlugin05 {}

impl McpPlugin06 {}

impl McpPlugin07 {}

impl McpPlugin08 {}

impl McpPlugin09 {}

impl McpPlugin10 {}
