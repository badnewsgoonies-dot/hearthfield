use bevy::prelude::*;

/// Shared high-level update phases used to make cross-domain system ordering explicit.
#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UpdatePhase {
    Input,
    Intent,
    Simulation,
    Reactions,
    Presentation,
}
