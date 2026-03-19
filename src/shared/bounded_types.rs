use ironclad::game_value;

#[game_value(min = 0, max = 100)]
pub struct Stamina(pub f32);

#[game_value(min = 0, max = 100)]
pub struct Health(pub f32);

#[game_value(min = 0, max = 999999)]
pub struct Gold(pub u32);

#[game_value(min = 0, max = 255)]
pub struct Happiness(pub u8);

#[game_value(min = 0, max = 1000)]
pub struct Friendship(pub u32);

#[game_value(min = 1, max = 99)]
pub struct StackSize(pub u8);

#[game_value(min = 0, max = 120)]
pub struct MineFloor(pub u8);

#[game_value(min = 0, max = 3)]
pub struct BuildingLevel(pub u8);
