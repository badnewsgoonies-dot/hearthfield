use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct OfficeWorker {
    pub energy: i32,
}

impl Default for OfficeWorker {
    fn default() -> Self {
        Self { energy: 100 }
    }
}

#[derive(Component)]
pub struct WorkerAvatar;

#[derive(Component)]
pub struct InboxAvatar;

#[allow(dead_code)]
#[derive(Component, Debug, Default)]
pub struct PlayerOfficeWorker;

#[allow(dead_code)]
#[derive(Component, Debug, Default)]
pub struct OfficeDesk;

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Copy)]
pub struct Interactable {
    pub interaction_id: &'static str,
}

#[allow(dead_code)]
impl Default for Interactable {
    fn default() -> Self {
        Self {
            interaction_id: "office_interaction",
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Default)]
pub enum NpcRole {
    Manager,
    #[default]
    Clerk,
    Analyst,
    Coordinator,
    Intern,
}

#[allow(dead_code)]
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct NpcCoworker {
    pub npc_id: u32,
    pub role: NpcRole,
}

#[allow(dead_code)]
impl NpcCoworker {
    pub const fn new(npc_id: u32, role: NpcRole) -> Self {
        Self { npc_id, role }
    }
}
