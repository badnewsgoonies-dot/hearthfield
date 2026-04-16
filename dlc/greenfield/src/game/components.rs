use bevy::prelude::*;

#[derive(Component, Debug, Default)]
pub struct GreenfieldRoot;

#[derive(Component, Debug, Default)]
pub struct PlayerMarker;

#[derive(Component, Debug, Default)]
pub struct CameraMarker;

#[derive(Component, Debug, Default)]
pub struct HudRoot;

#[derive(Component, Debug, Default)]
pub struct HudMainMenu;

#[derive(Component, Debug, Default)]
pub struct HudPause;

#[derive(Component, Debug, Default)]
pub struct RecordingMarker;

#[derive(Component, Debug, Default)]
pub struct DebugOverlay;

#[derive(Component, Debug, Default)]
pub struct SaveMarker;

#[derive(Component, Debug, Default)]
pub struct LoadMarker;

#[derive(Component, Debug, Default)]
pub struct AnimatedSprite;

#[derive(Component, Debug, Default)]
pub struct HudTimer;

#[derive(Component, Debug, Default)]
pub struct Enemy;

#[derive(Component, Debug, Default)]
pub struct Ally;

#[derive(Component, Debug, Default)]
pub struct Projectile;

#[derive(Component, Debug, Default)]
pub struct HealthBar;

#[derive(Component, Debug, Default)]
pub struct Chest;

#[derive(Component, Debug, Default)]
pub struct Loot;

#[derive(Component, Debug, Default)]
pub struct Portal;

#[derive(Component, Debug, Default)]
pub struct Checkpoint;

#[derive(Component, Debug, Default)]
pub struct Spawner;

#[derive(Component, Debug, Default)]
pub struct Goal;

#[derive(Component, Debug, Default)]
pub struct Obstacle;

#[derive(Component, Debug, Default)]
pub struct Ammo;

#[derive(Component, Debug, Default)]
pub struct Door;

#[derive(Component, Debug, Default)]
pub struct Key;

#[derive(Component, Debug, Default)]
pub struct Health;

#[derive(Component, Debug, Default)]
pub struct Mana;
