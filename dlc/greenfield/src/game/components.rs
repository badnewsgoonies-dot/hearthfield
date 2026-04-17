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

#[derive(Component, Debug, Default)]
pub struct Shield;

#[derive(Component, Debug, Default)]
pub struct Weapon;

#[derive(Component, Debug, Default)]
pub struct Armor;

#[derive(Component, Debug, Default)]
pub struct Potion;

#[derive(Component, Debug, Default)]
pub struct Scroll;

#[derive(Component, Debug, Default)]
pub struct Gem;

#[derive(Component, Debug, Default)]
pub struct Relic;

#[derive(Component, Debug, Default)]
pub struct Keystone;

#[derive(Component, Debug, Default)]
pub struct Rune;

#[derive(Component, Debug, Default)]
pub struct Sigil;

#[derive(Component, Debug, Default)]
pub struct Totem;

#[derive(Component, Debug, Default)]
pub struct Trinket;

#[derive(Component, Debug, Default)]
pub struct Amulet;

#[derive(Component, Debug, Default)]
pub struct Ring;

#[derive(Component, Debug, Default)]
pub struct Charm;

#[derive(Component, Debug, Default)]
pub struct BenchmarkTag;

#[derive(Component, Debug, Default)]
pub struct McpOnlyProbeTag;

#[derive(Component, Debug, Default)]
pub struct McpBatchAlpha;

#[derive(Component, Debug, Default)]
pub struct McpBatchBeta;

#[derive(Component, Debug, Default)]
pub struct McpScaleTag01;

#[derive(Component, Debug, Default)]
pub struct McpScaleTag02;

#[derive(Component, Debug, Default)]
pub struct McpScaleTag03;

#[derive(Component, Debug, Default)]
pub struct McpScaleTag04;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag01;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag02;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag03;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag04;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag05;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag06;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag07;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag08;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag09;

#[derive(Component, Debug, Default, Clone)]
pub struct BigBatchTag10;

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct01 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct02 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct03 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct04 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct05 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct06 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct07 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct08 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct09 {
    pub value: u32,
    pub tag: u64,
}

#[derive(Component, Debug, Default, Clone)]
pub struct McpStruct10 {
    pub value: u32,
    pub tag: u64,
}

pub enum McpEnum {    Variant01 { count: u32 },
    Variant02 { count: u32 },
    Variant03 { count: u32 },
    Variant04 { count: u32 },
    Variant05 { count: u32 },
    Variant06 { count: u32 },
    Variant07 { count: u32 },
    Variant08 { count: u32 },
    Variant09 { count: u32 },
    Variant10 { count: u32 },
    Variant11 { count: u32 },
    Variant12 { count: u32 },
    Variant13 { count: u32 },
}

impl McpStruct01 {    pub fn new_01() -> Self { Self::default() }
}

impl McpStruct02 {    pub fn new_02() -> Self { Self::default() }
}

impl McpStruct03 {    pub fn new_03() -> Self { Self::default() }
}

impl McpStruct04 {    pub fn new_04() -> Self { Self::default() }
}

impl McpStruct05 {    pub fn new_05() -> Self { Self::default() }
}

impl McpStruct06 {    pub fn new_06() -> Self { Self::default() }
}

impl McpStruct07 {    pub fn new_07() -> Self { Self::default() }
}

impl McpStruct08 {    pub fn new_08() -> Self { Self::default() }
}

impl McpStruct09 {    pub fn new_09() -> Self { Self::default() }
}

impl McpStruct10 {    pub fn new_10() -> Self { Self::default() }
}

#[derive(Component, Debug, Default)]
pub struct TrialA3Comp;

#[derive(Component, Debug, Default)]
pub struct HeartbeatMarker;

#[derive(Component, Debug, Default)]
pub struct CombatWeapon;

#[derive(Component, Debug, Default)]
pub struct CombatTarget;

#[derive(Component, Debug, Default)]
pub struct CombatParticipant;

#[derive(Component, Debug, Default)]
pub struct ItemInstance;

#[derive(Component, Debug, Default)]
pub struct ItemMarker;

#[derive(Component, Debug, Default)]
pub struct InventorySlot;

#[derive(Component, Debug, Default)]
pub struct InventoryContainer;

#[derive(Component, Debug, Default)]
pub struct EquippedItem;

#[derive(Component, Debug, Default)]
pub struct StackableItem;

#[derive(Component, Debug, Default)]
pub struct ConsumableItem;

#[derive(Component, Debug, Default)]
pub struct CraftingBench;

#[derive(Component, Debug, Default)]
pub struct CraftingMaterial;

#[derive(Component, Debug, Default)]
pub struct CraftingRecipe;

#[derive(Component, Debug, Default)]
pub struct CraftingQueue;

#[derive(Component, Debug, Default)]
pub struct CraftingOutput;

#[derive(Component, Debug, Default)]
pub struct PlayerHealth;


#[derive(Component, Debug, Default)]
pub struct QuestMarker;

#[derive(Component, Debug, Default)]
pub struct QuestGiver;

#[derive(Component, Debug, Default)]
pub struct QuestObjective;

#[derive(Component, Debug, Default)]
pub struct MusicTrackTag;

#[derive(Component, Debug, Default)]
pub struct MusicLayerActive;

#[derive(Component, Debug, Default)]
pub struct CheckpointBeacon;

#[derive(Component, Debug, Default)]
pub struct CheckpointVisited;
