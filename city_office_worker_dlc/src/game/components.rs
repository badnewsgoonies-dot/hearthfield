use bevy::prelude::*;

#[derive(Component, Debug)]
pub struct OfficeWorker {
    pub energy: i32,
}

#[derive(Component)]
pub struct WorkerAvatar;

#[derive(Component)]
pub struct InboxAvatar;
